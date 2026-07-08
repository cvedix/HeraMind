//! Isolated extension loader
//!
//! This loader wraps native extensions in a process-isolated environment.
//! Extensions loaded through this loader run in a separate process and
//! communicate via IPC, ensuring that extension crashes cannot affect
//! the main HeraMind process.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::NativeExtensionMetadataLoader;
use crate::extension::isolated::{IsolatedExtension, IsolatedExtensionConfig};
use crate::extension::system::{ExtensionMetadata, ExtensionMetricValue};
use crate::extension::types::{ExtensionError, Result};
use serde::Deserialize;

/// Manifest.json structure for loading metadata
#[derive(Debug, Clone, Deserialize)]
struct ExtensionManifest {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<String>,
}

/// Configuration for the isolated loader
#[derive(Debug, Clone)]
pub struct IsolatedLoaderConfig {
    /// Base configuration for isolated extensions
    pub isolated_config: IsolatedExtensionConfig,
    /// Whether to use isolated mode by default
    pub use_isolated_by_default: bool,
    /// Extensions that should always run in isolated mode
    pub force_isolated: Vec<String>,
}

impl Default for IsolatedLoaderConfig {
    fn default() -> Self {
        Self {
            isolated_config: IsolatedExtensionConfig::default(),
            use_isolated_by_default: true,
            force_isolated: Vec::new(),
        }
    }
}

/// Loader for isolated extensions
pub struct IsolatedExtensionLoader {
    /// Metadata loader for extension discovery (uses sidecar JSON, no dlopen)
    native_loader: NativeExtensionMetadataLoader,
    /// Configuration
    config: IsolatedLoaderConfig,
}

impl IsolatedExtensionLoader {
    /// Create a new isolated extension loader
    pub fn new(config: IsolatedLoaderConfig) -> Self {
        Self {
            native_loader: NativeExtensionMetadataLoader::new(),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(IsolatedLoaderConfig::default())
    }

    /// Check if an extension should run in isolated mode
    pub fn should_use_isolated(&self, extension_id: &str) -> bool {
        // Check force lists first
        if self.config.force_isolated.iter().any(|s| s == extension_id) {
            return true;
        }
        self.config.use_isolated_by_default
    }

    /// Load metadata from manifest.json in the extension directory
    fn load_metadata_from_manifest(path: &Path) -> Option<ExtensionMetadata> {
        // Try to find manifest.json in the extension directory
        let ext_dir = path.parent()?;
        let manifest_path = ext_dir.join("manifest.json");

        if !manifest_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&manifest_path).ok()?;
        let manifest: ExtensionManifest = serde_json::from_str(&content).ok()?;

        let version = semver::Version::parse(&manifest.version)
            .unwrap_or_else(|_| semver::Version::new(0, 1, 0));

        tracing::debug!(
            manifest_path = %manifest_path.display(),
            extension_id = %manifest.id,
            "Loaded metadata from manifest.json"
        );

        Some(ExtensionMetadata {
            id: manifest.id,
            name: manifest.name,
            version: version.to_string(),
            description: manifest.description,
            author: manifest.author,
            homepage: None,
            license: None,
            file_path: Some(path.to_path_buf()),
            config_parameters: None,
        })
    }

    /// Load metadata from sidecar JSON file (e.g., extension.dylib.json)
    fn load_metadata_from_sidecar(path: &Path) -> Option<ExtensionMetadata> {
        let sidecar_path = path.with_extension("json");

        if !sidecar_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&sidecar_path).ok()?;
        let manifest: ExtensionManifest = serde_json::from_str(&content).ok()?;

        let version = semver::Version::parse(&manifest.version)
            .unwrap_or_else(|_| semver::Version::new(0, 1, 0));

        tracing::debug!(
            sidecar_path = %sidecar_path.display(),
            extension_id = %manifest.id,
            "Loaded metadata from sidecar JSON"
        );

        Some(ExtensionMetadata {
            id: manifest.id,
            name: manifest.name,
            version: version.to_string(),
            description: manifest.description,
            author: manifest.author,
            homepage: None,
            license: None,
            file_path: Some(path.to_path_buf()),
            config_parameters: None,
        })
    }

    /// Load metadata from extension.json in the binaries directory
    /// This handles the .nep package format: binaries/{platform}/extension.json
    fn load_metadata_from_extension_json(path: &Path) -> Option<ExtensionMetadata> {
        // For binaries/{platform}/extension.dylib, look for binaries/{platform}/extension.json
        let sidecar_path = path.with_extension("json");

        if sidecar_path.exists() {
            if let Some(meta) = Self::load_metadata_from_sidecar(path) {
                return Some(meta);
            }
        }

        // Also try looking for a JSON file in the parent binaries directory
        // This handles legacy formats
        let ext_dir = path.parent()?;
        let parent_dir = ext_dir.parent()?; // Go up from darwin_aarch64 to binaries
        let ext_json = parent_dir.join("extension.json");

        if !ext_json.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&ext_json).ok()?;
        let manifest: ExtensionManifest = serde_json::from_str(&content).ok()?;

        let version = semver::Version::parse(&manifest.version)
            .unwrap_or_else(|_| semver::Version::new(0, 1, 0));

        tracing::debug!(
            ext_json_path = %ext_json.display(),
            extension_id = %manifest.id,
            "Loaded metadata from extension.json"
        );

        Some(ExtensionMetadata {
            id: manifest.id,
            name: manifest.name,
            version: version.to_string(),
            description: manifest.description,
            author: manifest.author,
            homepage: None,
            license: None,
            file_path: Some(path.to_path_buf()),
            config_parameters: None,
        })
    }

    /// Load an extension in isolated mode
    pub async fn load_isolated(&self, path: &Path) -> Result<Arc<IsolatedExtension>> {
        // IMPORTANT: We do NOT load the native library in the main process anymore.
        // This prevents file handle leaks that cause "Code Signature Invalid" crashes
        // when users uninstall and reinstall extensions.
        //
        // FFI validation is performed by the isolated extension-runner process at startup.
        // If the extension is incompatible, the runner will fail to start and report an error.
        //
        // Load metadata from manifest.json or sidecar JSON files instead.

        // Try to load metadata from manifest.json first (installed .nep package format)
        // Then try sidecar JSON (discovery format), finally try extension.json in binaries dir
        // Fallback to native loader for legacy/test extensions without JSON metadata
        let metadata = if let Some(manifest_meta) = Self::load_metadata_from_manifest(path) {
            tracing::debug!(
                path = %path.display(),
                extension_id = %manifest_meta.id,
                "Using manifest.json metadata"
            );
            manifest_meta
        } else if let Some(sidecar_meta) = Self::load_metadata_from_sidecar(path) {
            tracing::debug!(
                path = %path.display(),
                extension_id = %sidecar_meta.id,
                "Using sidecar JSON metadata"
            );
            sidecar_meta
        } else if let Some(ext_json_meta) = Self::load_metadata_from_extension_json(path) {
            tracing::debug!(
                path = %path.display(),
                extension_id = %ext_json_meta.id,
                "Using extension.json metadata"
            );
            ext_json_meta
        } else {
            // Fallback: use native loader for legacy/test extensions without JSON metadata
            // This is needed for test fixtures and development scenarios
            tracing::debug!(
                path = %path.display(),
                "No JSON metadata found, falling back to native loader (legacy/test mode)"
            );
            self.native_loader.load_metadata(path).await.map_err(|e| {
                ExtensionError::LoadFailed(format!(
                    "No metadata file found for extension at {} and native loader failed: {}. \
                     Expected one of: manifest.json, extension.json (sidecar), or binaries/*.json.",
                    path.display(),
                    e
                ))
            })?
        };

        tracing::debug!(
            extension_id = %metadata.id,
            path = %path.display(),
            "Loading extension in isolated mode"
        );

        // Create isolated extension wrapper
        let isolated =
            IsolatedExtension::new(&metadata.id, path, self.config.isolated_config.clone());

        // Start the extension process
        let start_time = std::time::Instant::now();
        isolated.start().await.map_err(|e| {
            ExtensionError::LoadFailed(format!(
                "Failed to start isolated extension '{}' after {:.1}s: {}",
                metadata.id,
                start_time.elapsed().as_secs_f64(),
                e
            ))
        })?;
        tracing::info!(
            extension_id = %metadata.id,
            elapsed_ms = start_time.elapsed().as_millis() as u64,
            "Extension process started successfully"
        );

        Ok(Arc::new(isolated))
    }

    /// Load an extension (decides mode based on configuration)
    pub async fn load(&self, path: &Path) -> Result<LoadedExtension> {
        // Extract metadata using safe JSON-based methods first, then fallback to native loader
        let metadata = if let Some(manifest_meta) = Self::load_metadata_from_manifest(path) {
            manifest_meta
        } else if let Some(sidecar_meta) = Self::load_metadata_from_sidecar(path) {
            sidecar_meta
        } else if let Some(ext_json_meta) = Self::load_metadata_from_extension_json(path) {
            ext_json_meta
        } else {
            // Fallback: use native loader for legacy/test extensions
            self.native_loader.load_metadata(path).await.map_err(|e| {
                ExtensionError::LoadFailed(format!(
                    "No metadata file found for extension at {} and native loader failed: {}",
                    path.display(),
                    e
                ))
            })?
        };

        if self.should_use_isolated(&metadata.id) {
            let isolated = self.load_isolated(path).await?;
            Ok(LoadedExtension::Isolated(isolated))
        } else {
            Err(ExtensionError::InvalidFormat(
                "In-process native loading has been removed; extensions must run isolated"
                    .to_string(),
            ))
        }
    }

    /// Discover extensions in a directory
    pub async fn discover(&self, dir: &Path) -> Vec<(PathBuf, ExtensionMetadata)> {
        // Use native loader for discovery
        self.native_loader.discover(dir).await
    }

    /// Get the configuration
    pub fn config(&self) -> &IsolatedLoaderConfig {
        &self.config
    }
}

/// Result of loading an extension
#[derive(Clone)]
pub enum LoadedExtension {
    /// Extension loaded in isolated process
    Isolated(Arc<IsolatedExtension>),
}

impl LoadedExtension {
    /// Get the extension ID
    pub async fn extension_id(&self) -> String {
        match self {
            Self::Isolated(isolated) => isolated.extension_id(),
        }
    }

    /// Check if this is an isolated extension
    pub fn is_isolated(&self) -> bool {
        true
    }

    /// Get metadata
    pub async fn metadata(&self) -> Option<ExtensionMetadata> {
        match self {
            Self::Isolated(isolated) => isolated.metadata().await,
        }
    }

    /// Execute a command
    pub async fn execute_command(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> std::result::Result<serde_json::Value, ExtensionError> {
        match self {
            Self::Isolated(isolated) => isolated
                .execute_command(command, args)
                .await
                .map_err(|e| ExtensionError::ExecutionFailed(e.to_string())),
        }
    }

    /// Produce metrics
    pub async fn produce_metrics(
        &self,
    ) -> std::result::Result<Vec<ExtensionMetricValue>, ExtensionError> {
        match self {
            Self::Isolated(isolated) => isolated
                .produce_metrics()
                .await
                .map_err(|e| ExtensionError::ExecutionFailed(e.to_string())),
        }
    }

    /// Health check
    pub async fn health_check(&self) -> std::result::Result<bool, ExtensionError> {
        match self {
            Self::Isolated(isolated) => isolated
                .health_check()
                .await
                .map_err(|e| ExtensionError::ExecutionFailed(e.to_string())),
        }
    }

    /// Stop the extension (only meaningful for isolated extensions)
    pub async fn stop(&self) -> std::result::Result<(), ExtensionError> {
        match self {
            Self::Isolated(isolated) => isolated
                .stop()
                .await
                .map_err(|e| ExtensionError::ExecutionFailed(e.to_string())),
        }
    }

    /// Check if the extension is alive
    pub fn is_alive(&self) -> bool {
        match self {
            Self::Isolated(isolated) => isolated.is_alive(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_config_default() {
        let config = IsolatedLoaderConfig::default();
        assert!(config.use_isolated_by_default);
        assert!(config.force_isolated.is_empty());
    }

    #[test]
    fn test_should_use_isolated() {
        let config = IsolatedLoaderConfig {
            use_isolated_by_default: true,
            force_isolated: vec!["dangerous-ext".to_string()],
            ..Default::default()
        };

        let loader = IsolatedExtensionLoader::new(config);

        assert!(loader.should_use_isolated("dangerous-ext"));
        assert!(loader.should_use_isolated("other-ext"));
    }
}
