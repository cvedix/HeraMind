//! Handlebars template rendering for data push payloads.

use crate::types::TemplateContext;
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::sync::Arc;

/// Thread-safe template renderer.
pub struct TemplateRenderer {
    registry: Arc<std::sync::Mutex<Handlebars<'static>>>,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        let mut registry = Handlebars::new();
        registry.register_escape_fn(handlebars::no_escape);
        // Register built-in helpers: json, timestamp_format
        registry.register_helper("json", Box::new(json_helper));
        registry.register_helper("timestamp_format", Box::new(timestamp_format_helper));
        Self {
            registry: Arc::new(std::sync::Mutex::new(registry)),
        }
    }

    /// Render a template string with the given context.
    /// If template is None or empty, returns the raw JSON of the context.
    pub fn render(&self, template: &Option<String>, context: &TemplateContext) -> Result<String> {
        match template {
            Some(tmpl) if !tmpl.is_empty() => {
                let data = json!({
                    "source_id": context.source_id,
                    "value": context.value,
                    "timestamp": context.timestamp,
                    "metadata": context.metadata,
                });
                let registry = self.registry.lock().unwrap();
                let rendered = registry.render_template(tmpl, &data)?;
                Ok(rendered)
            }
            _ => {
                // Default: render as JSON
                Ok(serde_json::to_string(&json!({
                    "source_id": context.source_id,
                    "value": context.value,
                    "timestamp": context.timestamp,
                    "metadata": context.metadata,
                }))?)
            }
        }
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// {{json value}} helper - serializes value to JSON string.
fn json_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h
        .param(0)
        .ok_or_else(|| handlebars::RenderErrorReason::ParamNotFoundForIndex("json", 0))?;
    let json_val = param.value();
    out.write(&serde_json::to_string(json_val).unwrap_or_default())?;
    Ok(())
}

/// {{timestamp_format timestamp}} helper - formats unix timestamp to ISO 8601.
fn timestamp_format_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).ok_or_else(|| {
        handlebars::RenderErrorReason::ParamNotFoundForIndex("timestamp_format", 0)
    })?;
    let ts = param.value().as_i64().unwrap_or(0);
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .unwrap_or_default()
        .to_rfc3339();
    out.write(&dt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_default_json() {
        let renderer = TemplateRenderer::new();
        let ctx = TemplateContext {
            source_id: "device:s1:temp".to_string(),
            value: json!(25.5),
            timestamp: 1700000000,
            metadata: None,
        };
        let result = renderer.render(&None, &ctx).unwrap();
        assert!(result.contains("device:s1:temp"));
        assert!(result.contains("25.5"));
    }

    #[test]
    fn test_render_custom_template() {
        let renderer = TemplateRenderer::new();
        let ctx = TemplateContext {
            source_id: "device:s1:temp".to_string(),
            value: json!(25.5),
            timestamp: 1700000000,
            metadata: None,
        };
        let template = Some("Temperature: {{value}} from {{source_id}}".to_string());
        let result = renderer.render(&template, &ctx).unwrap();
        assert_eq!(result, "Temperature: 25.5 from device:s1:temp");
    }
}
