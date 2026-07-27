//! Extension hardware variant detection + fallback chain.
//!
//! Single source of truth for selecting the right extension build variant
//! (cpu / cuda / jetson) on the **download side** (marketplace metadata
//! `builds` keys). The `.nep` internal structure is variant-agnostic —
//! variant discrimination lives only in marketplace `builds` keys and
//! release filenames.

use std::path::Path;

/// Hardware variant of the current device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Cpu,
    Cuda,
    Jetson,
}

impl Variant {
    /// Suffix appended to a hyphen-format platform key (matches metadata `builds` keys).
    /// `Cpu` has no suffix (the base platform key itself).
    pub fn suffix_hyphen(&self) -> Option<&'static str> {
        match self {
            Variant::Cpu => None,
            Variant::Cuda => Some("cuda"),
            Variant::Jetson => Some("jetson"),
        }
    }
}

/// Fallback candidate keys, most specific → most general (no `wasm`; caller appends it).
/// Examples:
///   ("linux-aarch64", Jetson) → ["linux-aarch64-jetson", "linux-aarch64"]
///   ("linux-aarch64", Cpu)    → ["linux-aarch64"]
pub fn fallback_keys(base_hyphen: &str, variant: Variant) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(suffix) = variant.suffix_hyphen() {
        keys.push(format!("{}-{}", base_hyphen, suffix));
    }
    keys.push(base_hyphen.to_string());
    keys
}

/// Parse the `HERAMIND_EXTENSION_VARIANT` override value. Returns:
/// - `Some(Variant)` for valid cpu/cuda/jetson
/// - `None` for `auto` / missing / invalid (caller logs invalid)
fn parse_override(raw: Option<&str>) -> Option<Variant> {
    match raw.map(str::trim) {
        Some("cpu") => Some(Variant::Cpu),
        Some("cuda") => Some(Variant::Cuda),
        Some("jetson") => Some(Variant::Jetson),
        // "auto" / None / invalid → no override
        _ => None,
    }
}

/// Pure, testable classifier. Inputs are passed in so tests don't touch
/// env vars or the filesystem. Detection precedence:
///   1. valid override env (cpu/cuda/jetson)
///   2. non-linux → Cpu (variants only meaningful on linux)
///   3. aarch64-linux + jetson marker → Jetson
///   4. nvidia-smi available (any linux arch) → Cuda
///   5. else → Cpu
pub fn classify_variant(
    os: &str,
    arch: &str,
    jetson_marker_exists: bool,
    nvidia_smi_ok: bool,
    override_env: Option<&str>,
) -> Variant {
    // Log + ignore invalid override values (treat as auto).
    if let Some(raw) = override_env {
        let trimmed = raw.trim();
        if !matches!(trimmed, "cpu" | "cuda" | "jetson" | "auto" | "") {
            tracing::warn!(
                "Invalid HERAMIND_EXTENSION_VARIANT={:?}, ignoring (expected cpu|cuda|jetson|auto)",
                trimmed
            );
        }
    }
    if let Some(v) = parse_override(override_env) {
        return v;
    }
    if os != "linux" {
        return Variant::Cpu;
    }
    if arch == "aarch64" && jetson_marker_exists {
        return Variant::Jetson;
    }
    if nvidia_smi_ok {
        return Variant::Cuda;
    }
    Variant::Cpu
}

/// Path checked for Jetson detection. Overridable for tests via env.
fn jetson_marker_path() -> String {
    std::env::var("HERAMIND_JETSON_MARKER_PATH")
        .unwrap_or_else(|_| "/etc/nv_tegra_release".to_string())
}

/// Best-effort `nvidia-smi` availability check. Failure → false (→ Cpu).
fn nvidia_smi_available() -> bool {
    // Match the pattern in stats.rs::detect_gpus (Command::new + success check).
    // Guarded: any error (binary missing / non-zero exit) → false.
    match std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

/// Detect the current device's hardware variant. OnceLock-cached at process
/// scope (only one nvidia-smi fork per process). Best-effort: any detection
/// failure degrades silently to Cpu and never blocks install/download.
pub fn detect_variant() -> Variant {
    static CACHE: std::sync::OnceLock<Variant> = std::sync::OnceLock::new();
    *CACHE.get_or_init(|| {
        classify_variant(
            std::env::consts::OS,
            std::env::consts::ARCH,
            Path::new(&jetson_marker_path()).exists(),
            nvidia_smi_available(),
            std::env::var("HERAMIND_EXTENSION_VARIANT").ok().as_deref(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_hyphen_values() {
        assert_eq!(Variant::Cpu.suffix_hyphen(), None);
        assert_eq!(Variant::Cuda.suffix_hyphen(), Some("cuda"));
        assert_eq!(Variant::Jetson.suffix_hyphen(), Some("jetson"));
    }

    #[test]
    fn fallback_keys_jetson() {
        let keys = fallback_keys("linux-aarch64", Variant::Jetson);
        assert_eq!(keys, vec!["linux-aarch64-jetson", "linux-aarch64"]);
    }

    #[test]
    fn fallback_keys_cuda() {
        let keys = fallback_keys("linux-x86_64", Variant::Cuda);
        assert_eq!(keys, vec!["linux-x86_64-cuda", "linux-x86_64"]);
    }

    #[test]
    fn fallback_keys_cpu_has_no_variant_suffix() {
        let keys = fallback_keys("linux-aarch64", Variant::Cpu);
        assert_eq!(keys, vec!["linux-aarch64"]);
    }

    #[test]
    fn fallback_keys_cpu_darwin() {
        let keys = fallback_keys("darwin-aarch64", Variant::Cpu);
        assert_eq!(keys, vec!["darwin-aarch64"]);
    }

    fn classify(
        os: &str,
        arch: &str,
        jetson_marker: bool,
        nvidia_smi: bool,
        ov: Option<&str>,
    ) -> Variant {
        super::classify_variant(os, arch, jetson_marker, nvidia_smi, ov)
    }

    #[test]
    fn override_takes_precedence() {
        // env override beats every physical signal
        assert_eq!(
            classify("linux", "aarch64", true, true, Some("cpu")),
            Variant::Cpu
        );
        assert_eq!(
            classify("linux", "x86_64", false, false, Some("jetson")),
            Variant::Jetson
        );
    }

    #[test]
    fn override_invalid_falls_back_to_auto() {
        // invalid override value → treated as auto (no override)
        assert_eq!(
            classify("linux", "aarch64", true, false, Some("bogus")),
            Variant::Jetson
        );
    }

    #[test]
    fn jetson_marker_on_aarch64_linux_detected() {
        assert_eq!(
            classify("linux", "aarch64", true, false, None),
            Variant::Jetson
        );
    }

    #[test]
    fn jetson_checked_before_cuda_so_nvidia_smi_does_not_misclassify() {
        // aarch64 + jetson marker + nvidia-smi present → still Jetson (checked first)
        assert_eq!(
            classify("linux", "aarch64", true, true, None),
            Variant::Jetson
        );
    }

    #[test]
    fn cuda_via_nvidia_smi_on_x86_64() {
        assert_eq!(
            classify("linux", "x86_64", false, true, None),
            Variant::Cuda
        );
    }

    #[test]
    fn plain_cpu_linux() {
        assert_eq!(
            classify("linux", "x86_64", false, false, None),
            Variant::Cpu
        );
    }

    #[test]
    fn non_linux_always_cpu_even_with_gpu_signals() {
        // macOS/Windows have no cuda/jetson variants
        assert_eq!(
            classify("macos", "aarch64", false, true, None),
            Variant::Cpu
        );
        assert_eq!(
            classify("windows", "x86_64", false, true, None),
            Variant::Cpu
        );
    }
}
