//! HeraMind Extension Package (.nep) parser and installer
//!
//! A .nep (HeraMind Extension Package) is a ZIP archive containing:
//! - manifest.json - Extension metadata
//! - binaries/ - Platform-specific extension binaries
//! - frontend/ - Frontend components and assets
//!
//! # Package Structure
//!
//! ```text
//! {extension-id}-{version}.nep
//! ├── manifest.json
//! ├── binaries/
//! │   ├── darwin_aarch64/
//! │   │   └── extension.dylib
//! │   ├── darwin_x86_64/
//! │   │   └── extension.dylib
//! │   ├── linux_amd64/
//! │   │   └── extension.so
//! │   ├── windows_amd64/
//! │   │   └── extension.dll
//! │   └── wasm/
//! │       ├── extension.wasm
//! │       └── extension.json
//! └── frontend/
//!     ├── dist/
//!     │   ├── bundle.js
//!     │   └── bundle.css
//!     └── assets/
//!         └── icons/
//! ```

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::extension::types::ExtensionError;

/// Extension package format identifier
pub const PACKAGE_FORMAT: &str = "heramind-extension-package";
/// Current supported ABI version (determines package format compatibility)
/// This single version number controls:
/// - Package format compatibility
/// - FFI interface compatibility
/// - Extension loading compatibility
pub const CURRENT_ABI_VERSION: u32 = 3;
/// Minimum supported ABI version
pub const MIN_ABI_VERSION: u32 = 3;

/// Parsed extension package
#[derive(Debug, Clone)]
pub struct ExtensionPackage {
    /// Package file path (if loaded from file)
    pub path: Option<PathBuf>,
    /// Parsed manifest
    pub manifest: ExtensionPackageManifest,
    /// Package file SHA256 checksum
    pub checksum: String,
    /// Package file size in bytes
    pub size: u64,
}

/// Extension package manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPackageManifest {
    /// Package format identifier
    pub format: String,
    /// Package format version (deprecated, use abi_version)
    /// Still accepted for backward compatibility
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format_version: Option<String>,

    /// ABI version - determines binary compatibility
    /// This is the primary version check for extension loading
    #[serde(default = "default_abi_version")]
    pub abi_version: u32,

    /// Extension metadata
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Minimum HeraMind version required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heramind: Option<NeomindRequirements>,

    /// Binary files for different platforms
    #[serde(default)]
    pub binaries: HashMap<String, String>,

    /// Frontend components
    /// Supports both string (e.g., "frontend/") and struct format for backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_frontend_opt", default)]
    pub frontend: Option<FrontendConfig>,

    /// Extension capabilities (metrics, commands)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,

    /// Permissions required
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Extension type (native, wasm, frontend-only)
    #[serde(default = "default_extension_type")]
    #[serde(rename = "type")]
    pub extension_type: String,
}

fn default_abi_version() -> u32 {
    CURRENT_ABI_VERSION
}

fn default_extension_type() -> String {
    "native".to_string()
}

/// HeraMind compatibility requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeomindRequirements {
    /// Minimum HeraMind version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

/// Frontend configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontendConfig {
    /// Dashboard components provided by this extension
    #[serde(default)]
    pub components: Vec<DashboardComponentDef>,
}

/// Intermediate type for deserializing frontend field that accepts both string and struct formats
/// This provides backward compatibility with older extension packages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrontendField {
    /// String format (e.g., "frontend/") - used in older extension packages
    String(String),
    /// Struct format with components
    Struct(FrontendConfig),
}

impl FrontendField {
    /// Convert to FrontendConfig, handling both formats
    pub fn into_config(self) -> FrontendConfig {
        match self {
            FrontendField::String(_) => FrontendConfig {
                components: Vec::new(),
            },
            FrontendField::Struct(config) => config,
        }
    }
}

/// Helper function to deserialize optional frontend field
fn deserialize_frontend_opt<'de, D>(deserializer: D) -> Result<Option<FrontendConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<FrontendField> = Option::deserialize(deserializer)?;
    Ok(opt.map(|f| f.into_config()))
}

/// Dashboard component definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardComponentDef {
    /// Component type ID
    #[serde(rename = "type")]
    pub component_type: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Component category
    pub category: String,
    /// Icon name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Path to bundled JS file (relative to frontend/)
    pub bundle_path: String,
    /// Exported component name
    pub export_name: String,
    /// Global variable name for the bundle (used for script tag loading)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_name: Option<String>,
    /// Size constraints
    #[serde(default)]
    pub size_constraints: SizeConstraints,
    /// Has data source configuration
    #[serde(default)]
    pub has_data_source: bool,
    /// Has display configuration
    #[serde(default)]
    pub has_display_config: bool,
    /// Has actions configuration
    #[serde(default)]
    pub has_actions: bool,
    /// Maximum data sources
    #[serde(default)]
    pub max_data_sources: usize,
    /// Configuration schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
    /// Data source schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source_schema: Option<serde_json::Value>,
    /// Default configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_config: Option<serde_json::Value>,
    /// Component variants
    #[serde(default)]
    pub variants: Vec<String>,
    /// Data binding configuration
    #[serde(default)]
    pub data_binding: DataBindingConfig,
}

/// Size constraints for dashboard components
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SizeConstraints {
    #[serde(default = "default_min")]
    pub min_w: usize,
    #[serde(default = "default_min")]
    pub min_h: usize,
    #[serde(default = "default_2")]
    pub default_w: usize,
    #[serde(default = "default_4")]
    pub default_h: usize,
    #[serde(default)]
    pub max_w: Option<usize>,
    #[serde(default)]
    pub max_h: Option<usize>,
    #[serde(default)]
    pub preserve_aspect: bool,
}

fn default_min() -> usize {
    1
}

fn default_2() -> usize {
    2
}

fn default_4() -> usize {
    4
}

/// Data binding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataBindingConfig {
    /// Extension metric to bind to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_metric: Option<String>,
    /// Extension command to execute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_command: Option<String>,
    /// Required fields from command result
    #[serde(default)]
    pub required_fields: Vec<String>,
}

/// Extension capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    #[serde(default)]
    pub metrics: Vec<MetricDescriptor>,
    #[serde(default)]
    pub commands: Vec<CommandDescriptor>,
}

/// Metric descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDescriptor {
    pub name: String,
    pub display_name: String,
    pub data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Command descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDescriptor {
    pub name: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Installation result
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Extension ID
    pub extension_id: String,
    /// Installed version
    pub version: String,
    /// Binary file path
    pub binary_path: PathBuf,
    /// Manifest file path
    pub manifest_path: PathBuf,
    /// Frontend directory (if any)
    pub frontend_dir: Option<PathBuf>,
    /// Installed dashboard components
    pub components: Vec<DashboardComponentDef>,
    /// Package checksum
    pub checksum: String,
    /// Resources directory (if any) - contains models, configs, etc.
    pub resources_dir: Option<PathBuf>,
    /// Models directory (if any) - contains AI model files
    pub models_dir: Option<PathBuf>,
}

/// Extension package error
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Invalid package format: {0}")]
    InvalidFormat(String),

    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Incompatible version: required {required}, got {got}")]
    IncompatibleVersion { required: String, got: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<PackageError> for ExtensionError {
    fn from(err: PackageError) -> Self {
        ExtensionError::LoadFailed(err.to_string())
    }
}

impl ExtensionPackage {
    /// Load a package from a file
    pub async fn load(path: &Path) -> Result<Self, PackageError> {
        // Read file
        let mut file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let size = metadata.len();

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Calculate checksum
        let checksum = Self::calculate_checksum(&buffer);

        // Parse ZIP archive
        let cursor = Cursor::new(buffer);
        let mut archive = ZipArchive::new(cursor).map_err(|e| PackageError::Zip(e.to_string()))?;

        // Read manifest.json
        let manifest_content = Self::read_file_from_zip(&mut archive, "manifest.json")?;
        let manifest: ExtensionPackageManifest = serde_json::from_str(&manifest_content)?;

        // Validate manifest
        Self::validate_manifest(&manifest)?;

        Ok(Self {
            path: Some(path.to_path_buf()),
            manifest,
            checksum,
            size,
        })
    }

    /// Load a package from bytes
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PackageError> {
        let checksum = Self::calculate_checksum(&data);
        let size = data.len() as u64;

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor).map_err(|e| PackageError::Zip(e.to_string()))?;

        let manifest_content = Self::read_file_from_zip(&mut archive, "manifest.json")?;
        let manifest: ExtensionPackageManifest = serde_json::from_str(&manifest_content)?;

        Self::validate_manifest(&manifest)?;

        Ok(Self {
            path: None,
            manifest,
            checksum,
            size,
        })
    }

    /// Validate the manifest
    fn validate_manifest(manifest: &ExtensionPackageManifest) -> Result<(), PackageError> {
        if manifest.format != PACKAGE_FORMAT {
            return Err(PackageError::InvalidFormat(format!(
                "Expected format '{}', got '{}'",
                PACKAGE_FORMAT, manifest.format
            )));
        }

        // Check ABI version compatibility (primary version check)
        if manifest.abi_version < MIN_ABI_VERSION || manifest.abi_version > CURRENT_ABI_VERSION {
            return Err(PackageError::IncompatibleVersion {
                required: format!("{}-{}", MIN_ABI_VERSION, CURRENT_ABI_VERSION),
                got: manifest.abi_version.to_string(),
            });
        }

        // Validate extension ID
        if manifest.id.is_empty() {
            return Err(PackageError::InvalidManifest(
                "Extension ID is required".to_string(),
            ));
        }

        // Validate version
        if manifest.version.is_empty() {
            return Err(PackageError::InvalidManifest(
                "Version is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate SHA256 checksum of data
    fn calculate_checksum(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Read a file from the ZIP archive
    fn read_file_from_zip<R: Read + std::io::Seek>(
        archive: &mut ZipArchive<R>,
        path: &str,
    ) -> Result<String, PackageError> {
        let mut file = archive
            .by_name(path)
            .map_err(|e| PackageError::MissingFile(format!("{}: {}", path, e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Get the binary path for the current platform
    /// Falls back to wasm if no native binary is available
    pub fn get_binary_path(&self) -> Option<String> {
        let platform = detect_platform();
        // First try native binary for current platform
        if let Some(path) = self.manifest.binaries.get(&platform) {
            return Some(path.clone());
        }
        // Fall back to wasm (universal platform)
        self.manifest.binaries.get("wasm").cloned()
    }

    /// Install the package to a target directory
    pub async fn install(&self, target_dir: &Path) -> Result<InstallResult, PackageError> {
        let ext_id = &self.manifest.id;
        let version = &self.manifest.version;

        // Create extension directory
        let ext_dir = target_dir.join(ext_id);
        tokio::fs::create_dir_all(&ext_dir).await?;

        // Load ZIP archive
        let data = if let Some(path) = &self.path {
            tokio::fs::read(path).await?
        } else {
            return Err(PackageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Package has no file path",
            )));
        };

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor).map_err(|e| PackageError::Zip(e.to_string()))?;

        // Extract manifest.json
        let manifest_path = ext_dir.join("manifest.json");
        self.extract_file(&mut archive, "manifest.json", &manifest_path)
            .await?;

        // Extract binary for current platform
        let binary_path = if let Some(rel_path) = self.get_binary_path() {
            let binary_file =
                ext_dir.join(PathBuf::from(&rel_path).file_name().unwrap_or_default());
            self.extract_file(&mut archive, &rel_path, &binary_file)
                .await?;

            // Create sidecar JSON file for all extension types (WASM and native)
            // This allows safe discovery without loading native libraries
            let sidecar_json = binary_file.with_extension("json");
            self.create_sidecar_json(&sidecar_json).await?;

            binary_file
        } else {
            let available_platforms: Vec<String> = self.manifest.binaries.keys().cloned().collect();
            return Err(PackageError::UnsupportedPlatform(format!(
                "{}. Available platforms: {}",
                detect_platform(),
                available_platforms.join(", ")
            )));
        };

        // Extract frontend directory if exists
        let frontend_dir = if self.manifest.frontend.is_some() {
            let frontend_path = ext_dir.join("frontend");
            self.extract_directory(&mut archive, "frontend/", &frontend_path)
                .await?;
            Some(frontend_path)
        } else {
            None
        };

        // ✨ Extract models directory if exists (AI model files)
        let models_path = ext_dir.join("models");
        let models_dir = if self
            .extract_directory(&mut archive, "models/", &models_path)
            .await
            .is_ok()
        {
            Some(models_path)
        } else {
            None
        };

        // ✨ Extract resources directory if exists (configs, assets, etc.)
        let resources_path = ext_dir.join("resources");
        let resources_dir = if self
            .extract_directory(&mut archive, "resources/", &resources_path)
            .await
            .is_ok()
        {
            Some(resources_path)
        } else {
            None
        };

        // 🔧 macOS: Re-sign all extracted dylibs after installation
        #[cfg(target_os = "macos")]
        {
            if let Some(binary_dir) = binary_path.parent() {
                Self::resign_dylibs_macos(binary_dir);
            }
        }

        // Get component definitions
        let components = self
            .manifest
            .frontend
            .as_ref()
            .map(|f| f.components.clone())
            .unwrap_or_default();

        Ok(InstallResult {
            extension_id: ext_id.clone(),
            version: version.clone(),
            binary_path,
            manifest_path,
            frontend_dir,
            components,
            checksum: self.checksum.clone(),
            resources_dir,
            models_dir,
        })
    }

    /// Install the package synchronously (for use in spawn_blocking)
    /// Takes raw package bytes since from_bytes() doesn't store them
    pub fn install_sync(data: &[u8], target_dir: &Path) -> Result<InstallResult, PackageError> {
        tracing::info!(
            "install_sync step 1/9: parse zip ({} bytes, target={})",
            data.len(),
            target_dir.display()
        );
        let cursor = Cursor::new(data.to_vec());
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            tracing::error!("install_sync FAILED step 1 (zip parse): {}", e);
            PackageError::Zip(e.to_string())
        })?;

        // Read manifest from archive
        let manifest_content = Self::read_file_from_zip(&mut archive, "manifest.json")?;
        let manifest: ExtensionPackageManifest = serde_json::from_str(&manifest_content)?;

        Self::validate_manifest(&manifest)?;

        let ext_id = &manifest.id;
        let version = &manifest.version;
        tracing::info!("install_sync step 2/9: manifest OK ({} v{})", ext_id, version);

        // Create extension directory
        let ext_dir = target_dir.join(ext_id);
        std::fs::create_dir_all(&ext_dir)?;

        // Extract manifest.json
        let manifest_path = ext_dir.join("manifest.json");
        Self::extract_file_sync(&mut archive, "manifest.json", &manifest_path)?;

        // Get binary path for current platform
        let platform = detect_platform();
        let binary_rel_path = manifest
            .binaries
            .get(&platform)
            .or_else(|| manifest.binaries.get("wasm"))
            .cloned()
            .ok_or_else(|| {
                let available_platforms: Vec<String> = manifest.binaries.keys().cloned().collect();
                tracing::error!(
                    "install_sync FAILED step 3: no binary for platform '{}' (have: {})",
                    platform,
                    available_platforms.join(", ")
                );
                PackageError::UnsupportedPlatform(format!(
                    "{}. Available platforms: {}",
                    platform,
                    available_platforms.join(", ")
                ))
            })?;

        tracing::info!(
            "install_sync step 3/9: platform {} -> {}",
            platform,
            binary_rel_path
        );

        // Extract binary and preserve directory structure.
        // Route through safe_join_within to reject `..` / absolute paths
        // coming from the manifest — this is the central zip-slip defense
        // (the async install path uses `file_name()` to flatten; this path
        // preserves subdirs for shared libraries, so we must guard instead).
        let binary_file = Self::safe_join_within(&ext_dir, &binary_rel_path)?;
        Self::extract_file_sync(&mut archive, &binary_rel_path, &binary_file)?;
        tracing::info!("install_sync step 4/9: binary extracted");

        // Extract all sibling files in the same directory as the binary
        // These are bundled shared libraries (e.g. libonnxruntime.dylib)
        if let Some(binary_dir) = std::path::Path::new(&binary_rel_path).parent() {
            let dir_prefix = if binary_dir.as_os_str().is_empty() {
                "".to_string()
            } else {
                format!("{}/", binary_dir.to_string_lossy())
            };
            let dest_dir = ext_dir.join(binary_dir);

            for i in 0..archive.len() {
                if let Ok(mut file) = archive.by_index(i) {
                    let name = file.name().to_string();
                    // Same directory, not the binary itself, not a directory entry
                    if name.starts_with(&dir_prefix)
                        && !name.ends_with('/')
                        && name != binary_rel_path
                        && !name[name.len() - 1..].starts_with('/')
                        && name.matches('/').count() == binary_rel_path.matches('/').count()
                    {
                        let dest = Self::safe_join_within(
                            &dest_dir,
                            name.trim_start_matches(&dir_prefix),
                        )?;
                        if let Some(parent) = dest.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        // macOS: if the destination already exists (re-install),
                        // code-signed files carry `com.apple.provenance` /
                        // embedded signatures that block File::create with
                        // EACCES. Remove first to clear xattrs + inode.
                        if dest.exists() {
                            let _ = std::fs::remove_file(&dest);
                        }
                        let mut out = std::fs::File::create(&dest)?;
                        std::io::copy(&mut file, &mut out)?;
                        tracing::info!("Extracted bundled library: {}", name);
                    }
                }
            }
        }

        // Create sidecar JSON file for all extension types (WASM and native)
        // This allows safe discovery without loading native libraries
        let sidecar_json = binary_file.with_extension("json");
        Self::create_sidecar_json_sync(&manifest, &sidecar_json)?;
        tracing::info!("install_sync step 5/9: sidecar json + sibling libs done");

        // Extract frontend directory if exists
        let frontend_dir = if manifest.frontend.is_some() {
            let frontend_path = ext_dir.join("frontend");
            Self::extract_directory_sync(&mut archive, "frontend/", &frontend_path)?;
            Some(frontend_path)
        } else {
            None
        };

        // Extract models directory if exists (for AI/ML extensions)
        let models_path = ext_dir.join("models");
        Self::extract_directory_sync(&mut archive, "models/", &models_path)?;

        // Extract assets directory if exists (for static assets)
        let assets_path = ext_dir.join("assets");
        Self::extract_directory_sync(&mut archive, "assets/", &assets_path)?;

        // Extract config directory if exists (for configuration files)
        let config_path = ext_dir.join("config");
        Self::extract_directory_sync(&mut archive, "config/", &config_path)?;
        tracing::info!("install_sync step 6/9: resource dirs done");

        // 🔧 macOS: Re-sign all extracted dylibs after installation.
        // When a .nep replaces an existing extension, macOS may cache the old code signature
        // for the file path/inode. Re-signing forces a fresh CDHash so the kernel accepts
        // the new binary when the extension-runner loads it via dlopen.
        // Without this, the runner gets SIGKILL (Code Signature Invalid) on launch.
        #[cfg(target_os = "macos")]
        {
            if let Some(binary_dir) = binary_file.parent() {
                tracing::info!("install_sync step 7/9: re-signing dylibs (macOS)");
                Self::resign_dylibs_macos(binary_dir);
                tracing::info!("install_sync step 7/9: resign done");
            }
        }

        // Get component definitions
        let components = manifest
            .frontend
            .as_ref()
            .map(|f| f.components.clone())
            .unwrap_or_default();

        // Calculate checksum
        let checksum = Self::calculate_checksum(data);

        // ✨ Determine which resource directories were extracted
        let models_dir = if models_path.exists() {
            Some(models_path)
        } else {
            None
        };

        let resources_dir = if assets_path.exists() || config_path.exists() {
            // Return assets as resources if either exists
            Some(assets_path)
        } else {
            None
        };

        tracing::info!("install_sync step 9/9: success ({} v{})", ext_id, version);
        Ok(InstallResult {
            extension_id: ext_id.clone(),
            version: version.clone(),
            binary_path: binary_file,
            manifest_path,
            frontend_dir,
            components,
            checksum,
            models_dir,
            resources_dir,
        })
    }

    /// Re-sign all .dylib files in the binary directory on macOS.
    /// This ensures macOS kernel code-signing cache is updated after
    /// overwriting existing dylibs during .nep installation.
    #[cfg(target_os = "macos")]
    fn resign_dylibs_macos(dir: &Path) {
        // Collect dylib paths first so we can dispatch them in parallel.
        // codesign is an external process — no shared mutable state, so
        // parallel execution is safe. This matters for bundles like
        // stream-player which ship 60+ FFmpeg dylibs: serial signing
        // takes 30-130s and blows past HTTP upload timeouts.
        let dylibs: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
            Ok(e) => e
                .flatten()
                .map(|entry| entry.path())
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("dylib"))
                .collect(),
            Err(_) => return,
        };

        if dylibs.is_empty() {
            return;
        }

        let start = std::time::Instant::now();
        let total = dylibs.len();

        // Spawn one thread per dylib. codesign is I/O-bound (fork + exec +
        // Mach IPC to the codesign daemon), so threads stay cheap even for
        // 60+ files. Capped implicitly by the OS scheduler.
        let successes = std::thread::scope(|s| {
            let handles: Vec<_> = dylibs
                .into_iter()
                .map(|path| {
                    s.spawn(move || {
                        match std::process::Command::new("codesign")
                            .args(["--force", "--sign", "-"])
                            .arg(&path)
                            .output()
                        {
                            Ok(output) if output.status.success() => true,
                            Ok(output) => {
                                tracing::warn!(
                                    "codesign failed for {}: {}",
                                    path.display(),
                                    String::from_utf8_lossy(&output.stderr).trim()
                                );
                                false
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "codesign failed for {}: {}",
                                    path.display(),
                                    e
                                );
                                false
                            }
                        }
                    })
                })
                .collect();

            handles
                .into_iter()
                .map(|h| h.join().unwrap_or(false))
                .filter(|&ok| ok)
                .count()
        });

        tracing::info!(
            "Re-signed {}/{} dylib(s) in {}ms (parallel)",
            successes,
            total,
            start.elapsed().as_millis()
        );
    }

    /// Extract a single file from the archive (synchronous)
    fn extract_file_sync<R: Read + std::io::Seek>(
        archive: &mut ZipArchive<R>,
        src_path: &str,
        dst_path: &Path,
    ) -> Result<(), PackageError> {
        let mut file = archive
            .by_name(src_path)
            .map_err(|e| PackageError::MissingFile(format!("{}: {}", src_path, e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        // Create parent directory
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // macOS: clear any existing file first — code-signed dylibs with
        // `com.apple.provenance` xattr reject fs::write with EACCES on
        // re-install. remove_file clears xattrs + inode.
        if dst_path.exists() {
            let _ = std::fs::remove_file(dst_path);
        }

        std::fs::write(dst_path, content)?;
        Ok(())
    }

    /// Resolve `rel_path` against `dst_dir`, rejecting any entry that would
    /// escape `dst_dir` via `..` traversal or absolute paths.
    ///
    /// This is the central zip-slip defense — both `extract_directory_sync`
    /// and `extract_directory` MUST route every zip entry through this helper.
    /// A malicious `.nep` package could otherwise include an entry such as
    /// `frontend/../../etc/cron.d/backdoor` and write outside the install dir.
    fn safe_join_within(dst_dir: &Path, rel_path: &str) -> Result<std::path::PathBuf, PackageError> {
        let mut resolved = dst_dir.to_path_buf();
        for component in std::path::Path::new(rel_path).components() {
            match component {
                std::path::Component::Normal(p) => resolved.push(p),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    return Err(PackageError::Zip(format!(
                        "path traversal rejected ( '..' in '{}')",
                        rel_path
                    )));
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    return Err(PackageError::Zip(format!(
                        "absolute path rejected in '{}'",
                        rel_path
                    )));
                }
            }
        }
        Ok(resolved)
    }

    /// Extract a directory from the archive (synchronous)
    fn extract_directory_sync<R: Read + std::io::Seek>(
        archive: &mut ZipArchive<R>,
        src_prefix: &str,
        dst_dir: &Path,
    ) -> Result<(), PackageError> {
        std::fs::create_dir_all(dst_dir)?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PackageError::Zip(format!("Failed to access file {}: {}", i, e)))?;

            let name = file.name().to_string();

            // Check if file starts with prefix
            if name.starts_with(src_prefix) && !name.ends_with('/') {
                // Remove prefix to get relative path
                let rel_path = name[src_prefix.len()..].to_string();
                let dst_path = Self::safe_join_within(dst_dir, &rel_path)?;

                // Create parent directory
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Extract file
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                std::fs::write(dst_path, content)?;
            }
        }

        Ok(())
    }

    /// Create a sidecar JSON file for WASM extensions (synchronous)
    fn create_sidecar_json_sync(
        manifest: &ExtensionPackageManifest,
        json_path: &Path,
    ) -> Result<(), PackageError> {
        use serde_json::json;

        // Build metrics array if capabilities exist
        let metrics = manifest
            .capabilities
            .as_ref()
            .map(|cap| {
                cap.metrics
                    .iter()
                    .map(|m| {
                        json!({
                            "name": m.name,
                            "display_name": m.display_name,
                            "data_type": m.data_type,
                            "unit": m.unit,
                            "min": m.min,
                            "max": m.max,
                            "required": false
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Build commands array if capabilities exist
        let commands = manifest.capabilities.as_ref().map(|cap| {
            cap.commands.iter().map(|c| {
                json!({
                    "name": c.name,
                    "display_name": c.display_name,
                    "payload_template": "",
                    "parameters": c.parameters.as_ref().map(|params| {
                        if let serde_json::Value::Object(props) = params {
                            if let Some(serde_json::Value::Object(props_map)) = props.get("properties") {
                                return props_map.iter().map(|(name, _)| {
                                    json!({
                                        "name": name,
                                        "display_name": name,
                                        "description": "",
                                        "param_type": "String",
                                        "required": false,
                                        "default_value": null,
                                        "min": null,
                                        "max": null,
                                        "options": []
                                    })
                                }).collect::<Vec<_>>();
                            }
                        }
                        Vec::<serde_json::Value>::new()
                    }).unwrap_or_default(),
                    "fixed_values": {},
                    "samples": [],
                    "description": ""
                })
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        let sidecar_data = json!({
            "id": manifest.id,
            "name": manifest.name,
            "version": manifest.version,
            "description": manifest.description,
            "author": manifest.author,
            "homepage": manifest.homepage,
            "license": manifest.license,
            "file_path": None::<String>,
            "metrics": metrics,
            "commands": commands
        });

        let content = serde_json::to_string_pretty(&sidecar_data).map_err(|e| {
            PackageError::InvalidManifest(format!("Failed to serialize sidecar JSON: {}", e))
        })?;

        std::fs::write(json_path, content).map_err(|e| {
            PackageError::Io(std::io::Error::other(format!(
                "Failed to write sidecar JSON: {}",
                e
            )))
        })?;

        Ok(())
    }

    /// Extract a single file from the archive
    async fn extract_file<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        src_path: &str,
        dst_path: &Path,
    ) -> Result<(), PackageError> {
        let mut file = archive
            .by_name(src_path)
            .map_err(|e| PackageError::MissingFile(format!("{}: {}", src_path, e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        // Create parent directory
        if let Some(parent) = dst_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(dst_path, content).await?;
        Ok(())
    }

    /// Extract a directory from the archive
    async fn extract_directory<R: Read + std::io::Seek>(
        &self,
        archive: &mut ZipArchive<R>,
        src_prefix: &str,
        dst_dir: &Path,
    ) -> Result<(), PackageError> {
        tokio::fs::create_dir_all(dst_dir).await?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| PackageError::Zip(format!("Failed to access file {}: {}", i, e)))?;

            let name = file.name().to_string();

            // Check if file starts with prefix
            if name.starts_with(src_prefix) && !name.ends_with('/') {
                // Remove prefix to get relative path
                let rel_path = name[src_prefix.len()..].to_string();
                let dst_path = Self::safe_join_within(dst_dir, &rel_path)?;

                // Create parent directory
                if let Some(parent) = dst_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                // Extract file
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                tokio::fs::write(dst_path, content).await?;
            }
        }

        Ok(())
    }

    /// Create a sidecar JSON file for WASM extensions
    /// This allows the WASM loader to find the metadata
    async fn create_sidecar_json(&self, json_path: &Path) -> Result<(), PackageError> {
        use serde_json::json;

        // Build metrics array if capabilities exist
        let metrics = self
            .manifest
            .capabilities
            .as_ref()
            .map(|cap| {
                cap.metrics
                    .iter()
                    .map(|m| {
                        json!({
                            "name": m.name,
                            "display_name": m.display_name,
                            "data_type": m.data_type,
                            "unit": m.unit,
                            "min": m.min,
                            "max": m.max,
                            "required": false
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Build commands array if capabilities exist
        let commands = self.manifest.capabilities.as_ref().map(|cap| {
            cap.commands.iter().map(|c| {
                json!({
                    "name": c.name,
                    "display_name": c.display_name,
                    "payload_template": "",
                    "parameters": c.parameters.as_ref().map(|params| {
                        if let serde_json::Value::Object(props) = params {
                            if let Some(serde_json::Value::Object(props_map)) = props.get("properties") {
                                return props_map.iter().map(|(name, _)| {
                                    json!({
                                        "name": name,
                                        "display_name": name,
                                        "description": "",
                                        "param_type": "String",
                                        "required": false,
                                        "default_value": null,
                                        "min": null,
                                        "max": null,
                                        "options": []
                                    })
                                }).collect::<Vec<_>>();
                            }
                        }
                        Vec::<serde_json::Value>::new()
                    }).unwrap_or_default(),
                    "fixed_values": {},
                    "samples": [],
                    "description": ""
                })
            }).collect::<Vec<_>>()
        }).unwrap_or_default();

        let sidecar_data = json!({
            "id": self.manifest.id,
            "name": self.manifest.name,
            "version": self.manifest.version,
            "description": self.manifest.description,
            "author": self.manifest.author,
            "homepage": self.manifest.homepage,
            "license": self.manifest.license,
            "file_path": self.path,
            "metrics": metrics,
            "commands": commands
        });

        let content = serde_json::to_string_pretty(&sidecar_data).map_err(|e| {
            PackageError::InvalidManifest(format!("Failed to serialize sidecar JSON: {}", e))
        })?;

        tokio::fs::write(json_path, content).await.map_err(|e| {
            PackageError::Io(std::io::Error::other(format!(
                "Failed to write sidecar JSON: {}",
                e
            )))
        })?;

        Ok(())
    }

    /// Uninstall an extension (remove its directory)
    pub async fn uninstall(install_dir: &Path, extension_id: &str) -> Result<(), PackageError> {
        let ext_dir = install_dir.join(extension_id);

        if ext_dir.exists() {
            tokio::fs::remove_dir_all(&ext_dir).await?;
        }

        Ok(())
    }
}

/// Platform format types used in the extension system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFormat {
    /// Hyphen format: "windows-x86_64", "darwin-aarch64"
    /// Used in marketplace metadata `builds` keys
    Hyphen,
    /// Underscore format: "windows_amd64", "darwin_aarch64"
    /// Used in .nep package `binaries` keys and filenames
    Underscore,
}

/// Detect the current platform
/// Returns platform string in underscore format (e.g., "darwin_aarch64")
/// to match the format used in extension packages
pub fn detect_platform() -> String {
    detect_platform_with_format(PlatformFormat::Underscore)
}

/// Detect the current platform with specified format
pub fn detect_platform_with_format(format: PlatformFormat) -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch, format) {
        // macOS
        ("macos", "aarch64", PlatformFormat::Hyphen) => "darwin-aarch64".to_string(),
        ("macos", "aarch64", PlatformFormat::Underscore) => "darwin_aarch64".to_string(),
        ("macos", "x86_64", PlatformFormat::Hyphen) => "darwin-x86_64".to_string(),
        ("macos", "x86_64", PlatformFormat::Underscore) => "darwin_x86_64".to_string(),
        // Linux
        ("linux", "x86_64", PlatformFormat::Hyphen) => "linux-x86_64".to_string(),
        ("linux", "x86_64", PlatformFormat::Underscore) => "linux_amd64".to_string(),
        ("linux", "aarch64", PlatformFormat::Hyphen) => "linux-aarch64".to_string(),
        ("linux", "aarch64", PlatformFormat::Underscore) => "linux_arm64".to_string(),
        // Windows
        ("windows", "x86_64", PlatformFormat::Hyphen) => "windows-x86_64".to_string(),
        ("windows", "x86_64", PlatformFormat::Underscore) => "windows_amd64".to_string(),
        ("windows", "aarch64", PlatformFormat::Hyphen) => "windows-aarch64".to_string(),
        ("windows", "aarch64", PlatformFormat::Underscore) => "windows_arm64".to_string(),
        // Fallback
        _ => match format {
            PlatformFormat::Hyphen => format!("{}-{}", os, arch),
            PlatformFormat::Underscore => format!("{}_{}", os, arch),
        },
    }
}

/// Convert platform format between hyphen and underscore
pub fn convert_platform_format(platform: &str, target_format: PlatformFormat) -> String {
    // Detect source format
    let (os, arch) = if platform.contains('-') {
        // Hyphen format: "windows-x86_64"
        let parts: Vec<&str> = platform.splitn(2, '-').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return platform.to_string();
        }
    } else if platform.contains('_') {
        // Underscore format: "windows_amd64"
        let parts: Vec<&str> = platform.splitn(2, '_').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return platform.to_string();
        }
    } else {
        return platform.to_string();
    };

    // Map arch names: x86_64 <-> amd64, aarch64 <-> arm64
    let normalized_arch = match arch {
        "x86_64" | "amd64" => ("x86_64", "amd64"),
        "aarch64" | "arm64" => ("aarch64", "arm64"),
        _ => (arch, arch),
    };

    match target_format {
        PlatformFormat::Hyphen => format!("{}-{}", os, normalized_arch.0),
        PlatformFormat::Underscore => format!("{}_{}", os, normalized_arch.1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_join_within_rejects_traversal() {
        use ExtensionPackage;
        let dst = std::path::Path::new("/tmp/ext-install");

        // Normal relative path resolves within dst_dir
        let ok = ExtensionPackage::safe_join_within(dst, "frontend/dist/bundle.js").unwrap();
        assert!(ok.starts_with(dst), "normal path should stay within dst_dir");

        // '..' in rel_path must be rejected (zip slip)
        assert!(
            ExtensionPackage::safe_join_within(dst, "../../etc/cron.d/backdoor").is_err(),
            "traversal via .. must be rejected"
        );
        // Mid-path '..' also rejected
        assert!(
            ExtensionPackage::safe_join_within(dst, "frontend/../../../etc/passwd").is_err(),
            "mid-path traversal must be rejected"
        );
        // Absolute path rejected
        assert!(
            ExtensionPackage::safe_join_within(dst, "/etc/passwd").is_err(),
            "absolute path must be rejected"
        );
        // '.' is allowed (no-op)
        assert!(
            ExtensionPackage::safe_join_within(dst, "frontend/./bundle.js").is_ok(),
            "current-dir component should be allowed"
        );
    }

    #[test]
    fn test_detect_platform() {
        let platform = detect_platform();
        println!("Detected platform: {}", platform);
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_format_constants() {
        assert_eq!(PACKAGE_FORMAT, "heramind-extension-package");
        assert_eq!(CURRENT_ABI_VERSION, 3);
        assert_eq!(MIN_ABI_VERSION, 3);
    }

    #[test]
    fn test_frontend_string_format() {
        // Test that frontend field accepts string format (backward compatibility)
        let json = r#"{
            "format": "heramind-extension-package",
            "abi_version": 3,
            "id": "test-extension",
            "name": "Test Extension",
            "version": "1.0.0",
            "frontend": "frontend/"
        }"#;

        let manifest: ExtensionPackageManifest =
            serde_json::from_str(json).expect("Failed to parse manifest with string frontend");

        assert_eq!(manifest.id, "test-extension");
        assert!(manifest.frontend.is_some());
        assert!(manifest.frontend.as_ref().unwrap().components.is_empty());
    }

    #[test]
    fn test_frontend_struct_format() {
        // Test that frontend field accepts struct format
        let json = r#"{
            "format": "heramind-extension-package",
            "abi_version": 3,
            "id": "test-extension",
            "name": "Test Extension",
            "version": "1.0.0",
            "frontend": {
                "components": [
                    {
                        "type": "card",
                        "name": "Weather Card",
                        "description": "A weather display card",
                        "category": "widget",
                        "bundle_path": "dist/bundle.js",
                        "export_name": "WeatherCard"
                    }
                ]
            }
        }"#;

        let manifest: ExtensionPackageManifest =
            serde_json::from_str(json).expect("Failed to parse manifest with struct frontend");

        assert_eq!(manifest.id, "test-extension");
        assert!(manifest.frontend.is_some());
        assert_eq!(manifest.frontend.as_ref().unwrap().components.len(), 1);
        assert_eq!(
            manifest.frontend.as_ref().unwrap().components[0].name,
            "Weather Card"
        );
    }

    #[test]
    fn test_frontend_null_format() {
        // Test that frontend field accepts null/missing
        let json = r#"{
            "format": "heramind-extension-package",
            "abi_version": 3,
            "id": "test-extension",
            "name": "Test Extension",
            "version": "1.0.0"
        }"#;

        let manifest: ExtensionPackageManifest =
            serde_json::from_str(json).expect("Failed to parse manifest without frontend");

        assert_eq!(manifest.id, "test-extension");
        assert!(manifest.frontend.is_none());
    }
}
