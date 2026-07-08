//! Structured payload template renderer.
//!
//! Replaces the legacy string-substitution approach with a JSON-aware
//! tree walker. Solves the five classes of bugs that `str.replace`
//! introduces:
//!
//!   1. Placeholder syntax drift (template says `${var}`, code looks for `${{var}}`)
//!   2. Quote-collision (renderer adds quotes on top of template's quotes)
//!   3. Type erasure (string `to_string` loses MetricValue variant info)
//!   4. JSON injection (partial escaping of `"`, `\`, control chars)
//!   5. Reactive validation (post-hoc `from_str` cannot pinpoint the bad param)
//!
//! ## Contract
//!
//! - Template is a JSON-shaped string with `${name}` placeholders.
//! - `${name}` may appear inside quoted strings (string interpolation)
//!   or in unquoted value positions (structural substitution).
//! - The `MetricValue` variant of each parameter determines the JSON
//!   type of the substituted value. Quoting in the template is **not**
//!   significant — `"${var}"` and `${var}` produce the same typed
//!   substitution. Template authors may keep quotes for readability
//!   or omit them; the result is identical.
//! - Binary values are rejected (no sensible JSON representation).
//! - Missing parameters produce `RenderError::PlaceholderNotFound(name)`
//!   so callers can pinpoint the bad input.
//!
//! ## Non-JSON payloads
//!
//! Some legacy devices (HASS, simple switches) use bare-string payloads
//! like `ON` or `OFF`. These bypass the JSON path entirely and fall
//! back to a simple string substitution that does NOT validate JSON.

use std::collections::HashMap;

use serde_json::Value;

use crate::mdl::MetricValue;

/// Error returned by [`render`].
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// Template declared as JSON (starts with `{` or `[`) but failed to parse.
    #[error("invalid JSON template: {0}")]
    InvalidTemplate(#[from] serde_json::Error),

    /// A `${name}` in the template has no matching entry in `params`.
    #[error("placeholder ${{{0}}} was not given a value")]
    PlaceholderNotFound(String),

    /// A parameter value cannot be rendered into JSON at all.
    /// Currently only `MetricValue::Binary` triggers this.
    #[error("parameter {name}: binary values are not supported in JSON payloads")]
    BinaryUnsupported { name: String },

    /// A parameter value cannot be stringified for substring interpolation.
    /// Arrays and binary values can't be losslessly inlined into a string.
    #[error(
        "parameter {name}: {variant} cannot be interpolated into a string — use the placeholder as the entire value instead"
    )]
    NotStringifiable { name: String, variant: &'static str },
}

// ---------------------------------------------------------------------------
// Sentinel format
// ---------------------------------------------------------------------------

/// Sentinel wrapper used during preprocessing.
///
/// After preprocessing, every `${name}` placeholder becomes either
/// `__PH:name__` (when it was inside a quoted string) or
/// `"__PH:name__"` (when it was in an unquoted value position). Both
/// forms are valid JSON, and the post-parse [`Value::String`] leaf can
/// be recognized unambiguously.
const SENTINEL_PREFIX: &str = "__PH:";
const SENTINEL_SUFFIX: &str = "__";

/// Test whether `s` is exactly `__PH:<name>__` and return `<name>`.
fn parse_exact_sentinel(s: &str) -> Option<&str> {
    let inner = s.strip_prefix(SENTINEL_PREFIX)?;
    let name = inner.strip_suffix(SENTINEL_SUFFIX)?;
    if is_valid_name(name) {
        Some(name)
    } else {
        None
    }
}

/// A valid placeholder name: ASCII identifier, dotted paths allowed.
fn is_valid_name(s: &str) -> bool {
    let mut iter = s.chars();
    match iter.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    for c in iter {
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '.') {
            return false;
        }
    }
    !s.is_empty()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Render a payload template with the given parameters.
///
/// See module docs for the contract. Returns the rendered payload as
/// UTF-8 bytes suitable for sending over MQTT/HTTP.
pub fn render(
    template: &str,
    params: &HashMap<String, MetricValue>,
) -> Result<Vec<u8>, RenderError> {
    let trimmed = template.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        render_json(template, params)
    } else {
        render_plain(template, params)
    }
}

// ---------------------------------------------------------------------------
// JSON path
// ---------------------------------------------------------------------------

fn render_json(
    template: &str,
    params: &HashMap<String, MetricValue>,
) -> Result<Vec<u8>, RenderError> {
    let preprocessed = preprocess_template(template);
    let mut tree: Value = serde_json::from_str(&preprocessed)?;
    walk_value(&mut tree, params)?;
    let bytes = serde_json::to_vec(&tree)?;
    Ok(bytes)
}

/// Phase 1: state-machine scan that rewrites `${name}` placeholders
/// into sentinel form so the result is valid JSON.
///
/// - Inside a JSON string literal: `${name}` → `__PH:name__`
/// - Outside a string literal:     `${name}` → `"__PH:name__"`
///
/// The state machine tracks string boundaries (respecting `\` escapes)
/// so it correctly handles templates whose parameter names happen to
/// contain quote-like characters (which they can't, given
/// [`is_valid_name`], but the discipline still matters for embedded
/// JSON-in-string scenarios).
fn preprocess_template(template: &str) -> String {
    let bytes: &[u8] = template.as_bytes();
    let mut out = String::with_capacity(template.len() + 16);
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i];

        if escaped {
            // Previous char was `\`; this char is taken literally.
            out.push(c as char);
            escaped = false;
            i += 1;
            continue;
        }

        if in_string {
            if c == b'\\' {
                out.push('\\');
                escaped = true;
                i += 1;
                continue;
            }
            if c == b'"' {
                in_string = false;
                out.push('"');
                i += 1;
                continue;
            }
            // Look for `${name}` inside the string.
            if let Some(end) = match_placeholder_at(&template[i..]) {
                let name = &template[i + 2..i + end];
                // Inside a string: emit bare sentinel (no added quotes).
                out.push_str(SENTINEL_PREFIX);
                out.push_str(name);
                out.push_str(SENTINEL_SUFFIX);
                i += end + 1;
                continue;
            }
            out.push(c as char);
            i += 1;
            continue;
        }

        // Outside any string literal.
        if c == b'"' {
            in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if let Some(end) = match_placeholder_at(&template[i..]) {
            let name = &template[i + 2..i + end];
            // Outside a string: wrap the sentinel as a JSON string so
            // the surrounding JSON remains parseable.
            out.push('"');
            out.push_str(SENTINEL_PREFIX);
            out.push_str(name);
            out.push_str(SENTINEL_SUFFIX);
            out.push('"');
            i += end + 1;
            continue;
        }
        out.push(c as char);
        i += 1;
    }

    out
}

/// If `s` starts with `${name}` (with a valid identifier name), return
/// the index of the closing `}` relative to the start of `s`.
/// Otherwise return `None`.
fn match_placeholder_at(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.len() < 4 || bytes[0] != b'$' || bytes[1] != b'{' {
        return None;
    }
    // Scan from index 2 until `}`.
    let mut end = 2;
    while end < bytes.len() && bytes[end] != b'}' {
        end += 1;
    }
    if end >= bytes.len() {
        return None; // no closing brace
    }
    let name = &s[2..end];
    if is_valid_name(name) {
        Some(end)
    } else {
        None
    }
}

/// Phase 3: recursively walk the JSON tree, replacing sentinel leaves.
fn walk_value(v: &mut Value, params: &HashMap<String, MetricValue>) -> Result<(), RenderError> {
    match v {
        Value::Object(map) => {
            for (_, child) in map.iter_mut() {
                walk_value(child, params)?;
            }
        }
        Value::Array(arr) => {
            for child in arr.iter_mut() {
                walk_value(child, params)?;
            }
        }
        Value::String(s) => {
            if let Some(new_value) = try_replace_leaf(s, params)? {
                *v = new_value;
            }
        }
        _ => {} // Number / Bool / Null — no placeholders possible
    }
    Ok(())
}

/// Given a JSON string leaf, decide whether it's a placeholder and
/// return the replacement Value.
fn try_replace_leaf(
    s: &str,
    params: &HashMap<String, MetricValue>,
) -> Result<Option<Value>, RenderError> {
    // Case 1: the entire string is exactly `__PH:name__`.
    //         → structural substitution; preserve the typed JSON value.
    if let Some(name) = parse_exact_sentinel(s) {
        let value = params
            .get(name)
            .ok_or_else(|| RenderError::PlaceholderNotFound(name.to_string()))?;
        return metric_to_json(name, value).map(Some);
    }

    // Case 2: the string contains one or more sentinels as substrings.
    //         → string interpolation; stringify each value.
    if !s.contains(SENTINEL_PREFIX) {
        return Ok(None);
    }
    let interpolated = interpolate_substrings(s, params)?;
    Ok(Some(Value::String(interpolated)))
}

/// Replace every `__PH:name__` substring inside `s` with the stringified
/// form of the corresponding parameter. Used for substring interpolation
/// (e.g. `"/users/${user_id}/info"` → `"/users/42/info"`).
fn interpolate_substrings(
    s: &str,
    params: &HashMap<String, MetricValue>,
) -> Result<String, RenderError> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;

    while let Some(prefix_start) = rest.find(SENTINEL_PREFIX) {
        out.push_str(&rest[..prefix_start]);
        let after_prefix = &rest[prefix_start + SENTINEL_PREFIX.len()..];

        // Find the suffix.
        let suffix_pos = after_prefix
            .find(SENTINEL_SUFFIX)
            .ok_or_else(|| RenderError::InvalidTemplate(serde::de::Error::custom(
                format!("malformed sentinel in template leaf: {}", rest),
            )))?;
        let name = &after_prefix[..suffix_pos];
        if !is_valid_name(name) {
            return Err(RenderError::InvalidTemplate(serde::de::Error::custom(
                format!("invalid placeholder name in template leaf: {}", name),
            )));
        }
        let value = params
            .get(name)
            .ok_or_else(|| RenderError::PlaceholderNotFound(name.to_string()))?;
        out.push_str(&stringify_for_interpolation(name, value)?);

        rest = &after_prefix[suffix_pos + SENTINEL_SUFFIX.len()..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Convert a `MetricValue` into the matching `serde_json::Value`.
///
/// Used for **structural substitution** (Case 1). The JSON type is
/// determined entirely by the `MetricValue` variant — the template's
/// quoting is not consulted. This is the root fix for the
/// "double-quoting" bug in the legacy renderer.
fn metric_to_json(name: &str, v: &MetricValue) -> Result<Value, RenderError> {
    match v {
        MetricValue::Integer(i) => Ok(Value::Number((*i).into())),
        MetricValue::Float(f) => match serde_json::Number::from_f64(*f) {
            Some(n) => Ok(Value::Number(n)),
            None => Ok(Value::Null), // NaN / Infinity → null per JSON spec
        },
        MetricValue::String(s) => Ok(Value::String(s.clone())),
        MetricValue::Boolean(b) => Ok(Value::Bool(*b)),
        MetricValue::Null => Ok(Value::Null),
        MetricValue::Array(arr) => {
            let mut json_arr = Vec::with_capacity(arr.len());
            for (idx, child) in arr.iter().enumerate() {
                let child_name = format!("{}[{}]", name, idx);
                json_arr.push(metric_to_json(&child_name, child)?);
            }
            Ok(Value::Array(json_arr))
        }
        MetricValue::Binary(_) => Err(RenderError::BinaryUnsupported {
            name: name.to_string(),
        }),
    }
}

/// Stringify a `MetricValue` for substring interpolation.
///
/// Strings/numbers/booleans/null become their natural string form.
/// Arrays and binary values are rejected — they can't be losslessly
/// inlined into a larger string; the template author should use the
/// placeholder as the *entire* value instead.
fn stringify_for_interpolation(name: &str, v: &MetricValue) -> Result<String, RenderError> {
    match v {
        MetricValue::String(s) => Ok(s.clone()),
        MetricValue::Integer(i) => Ok(i.to_string()),
        MetricValue::Float(f) => Ok(f.to_string()),
        MetricValue::Boolean(b) => Ok(b.to_string()),
        MetricValue::Null => Ok("null".to_string()),
        MetricValue::Array(_) => Err(RenderError::NotStringifiable {
            name: name.to_string(),
            variant: "array",
        }),
        MetricValue::Binary(_) => Err(RenderError::NotStringifiable {
            name: name.to_string(),
            variant: "binary",
        }),
    }
}

// ---------------------------------------------------------------------------
// Plain (non-JSON) fallback path
// ---------------------------------------------------------------------------

fn render_plain(
    template: &str,
    params: &HashMap<String, MetricValue>,
) -> Result<Vec<u8>, RenderError> {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;

    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after_dollar_brace = &rest[start + 2..];
        let end_rel = after_dollar_brace
            .find('}')
            .ok_or_else(|| RenderError::PlaceholderNotFound("${…".to_string()))?;
        let name = &after_dollar_brace[..end_rel];
        if !is_valid_name(name) {
            return Err(RenderError::PlaceholderNotFound(name.to_string()));
        }
        let value = params
            .get(name)
            .ok_or_else(|| RenderError::PlaceholderNotFound(name.to_string()))?;
        // For plain payloads there's no JSON context, so the natural
        // string form is the only sensible choice.
        out.push_str(&stringify_for_interpolation(name, value)?);
        rest = &after_dollar_brace[end_rel + 1..];
    }
    out.push_str(rest);
    Ok(out.into_bytes())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn param(name: &str, v: MetricValue) -> (String, MetricValue) {
        (name.to_string(), v)
    }

    // ----- NE301 contract tests (real template shape) -----

    #[test]
    fn ne301_capture_command_renders_correctly() {
        // Verbatim from crates/heramind-storage/src/builtin_types/ne301_camera.json
        // NE301 capture is a zero-param command — just cmd + request_id.
        // request_id is auto-injected by DeviceService::build_command_payload
        // before this renderer sees it.
        let template = r#"{"cmd": "capture", "request_id": "${request_id}"}"#;
        let params: HashMap<String, MetricValue> =
            [param("request_id", MetricValue::String("req-abc-123".into()))]
                .into_iter()
                .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");

        assert_eq!(parsed["cmd"], "capture");
        assert_eq!(parsed["request_id"], "req-abc-123");
        // The actual protocol has NO params field — make sure we don't
        // accidentally inject one.
        assert!(
            parsed.get("params").is_none(),
            "capture command must not have a params field"
        );
    }

    #[test]
    fn ne301_sleep_command_renders_correctly() {
        let template = r#"{"cmd": "sleep", "request_id": "${request_id}", "params": {"duration_sec": ${duration_sec}}}"#;
        let params: HashMap<String, MetricValue> = [
            param("request_id", MetricValue::String("req-002".into())),
            param("duration_sec", MetricValue::Integer(30)),
        ]
        .into_iter()
        .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");

        assert_eq!(parsed["cmd"], "sleep");
        assert_eq!(parsed["request_id"], "req-002");
        assert_eq!(parsed["params"]["duration_sec"], 30);
    }

    // ----- Placeholder syntax & quote-collision regression -----

    #[test]
    fn quoted_string_placeholder_does_not_double_quote() {
        // The legacy renderer would have produced "request_id": ""req-1""
        // here — this test pins the fix.
        let template = r#"{"request_id": "${request_id}"}"#;
        let params: HashMap<String, MetricValue> =
            [param("request_id", MetricValue::String("req-1".into()))]
                .into_iter()
                .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(parsed["request_id"], "req-1");
    }

    #[test]
    fn unquoted_integer_placeholder_becomes_number() {
        let template = r#"{"value": ${value}}"#;
        let params: HashMap<String, MetricValue> =
            [param("value", MetricValue::Integer(42))]
                .into_iter()
                .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(parsed["value"], 42);
    }

    #[test]
    fn unquoted_boolean_placeholder_becomes_bool() {
        let template = r#"{"enabled": ${enabled}}"#;
        let params: HashMap<String, MetricValue> =
            [param("enabled", MetricValue::Boolean(true))]
                .into_iter()
                .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(parsed["enabled"], true);
    }

    // ----- Substring interpolation -----

    #[test]
    fn substring_interpolation_inside_larger_string() {
        let template = r#"{"path": "/users/${user_id}/info"}"#;
        let params: HashMap<String, MetricValue> =
            [param("user_id", MetricValue::Integer(42))]
                .into_iter()
                .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(parsed["path"], "/users/42/info");
    }

    #[test]
    fn multiple_substring_interpolations_in_one_string() {
        let template = r#"{"url": "https://${host}:${port}/${path}"}"#;
        let params: HashMap<String, MetricValue> = [
            param("host", MetricValue::String("example.com".into())),
            param("port", MetricValue::Integer(8080)),
            param("path", MetricValue::String("api/v1".into())),
        ]
        .into_iter()
        .collect();

        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert_eq!(parsed["url"], "https://example.com:8080/api/v1");
    }

    // ----- Error cases -----

    #[test]
    fn missing_parameter_returns_specific_error() {
        let template = r#"{"request_id": "${request_id}"}"#;
        let params = HashMap::new();
        let err = render(template, &params).unwrap_err();
        match err {
            RenderError::PlaceholderNotFound(name) => assert_eq!(name, "request_id"),
            other => panic!("expected PlaceholderNotFound, got {:?}", other),
        }
    }

    #[test]
    fn binary_value_is_rejected() {
        let template = r#"{"data": ${data}}"#;
        let params: HashMap<String, MetricValue> =
            [param("data", MetricValue::Binary(vec![1, 2, 3]))]
                .into_iter()
                .collect();
        let err = render(template, &params).unwrap_err();
        assert!(matches!(err, RenderError::BinaryUnsupported { .. }));
    }

    #[test]
    fn malformed_template_returns_invalid_template_error() {
        let template = r#"{"cmd": "broken"#; // missing close
        let params = HashMap::new();
        assert!(matches!(
            render(template, &params),
            Err(RenderError::InvalidTemplate(_))
        ));
    }

    // ----- Plain (non-JSON) path -----

    #[test]
    fn plain_payload_string_substitution() {
        // HASS-style bare-string payloads.
        let template = "ON";
        let params = HashMap::new();
        let bytes = render(template, &params).expect("render");
        assert_eq!(bytes, b"ON");
    }

    #[test]
    fn plain_payload_with_substring_placeholder() {
        let template = "CMD:${action}";
        let params: HashMap<String, MetricValue> =
            [param("action", MetricValue::String("reset".into()))]
                .into_iter()
                .collect();
        let bytes = render(template, &params).expect("render");
        assert_eq!(bytes, b"CMD:reset");
    }

    // ----- Empty template edge case -----

    #[test]
    fn empty_json_object_template() {
        let template = "{}";
        let params = HashMap::new();
        let bytes = render(template, &params).expect("render");
        let parsed: Value = serde_json::from_slice(&bytes).expect("valid JSON");
        assert!(parsed.is_object());
        assert_eq!(parsed.as_object().unwrap().len(), 0);
    }
}
