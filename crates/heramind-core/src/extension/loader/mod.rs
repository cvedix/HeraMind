//! Extension loaders for native and isolated extensions.
//!
//! WASM extensions are now handled by the extension-runner process,
//! which uses wasmtime directly for execution. Native binaries are
//! discovered via metadata only and executed through the runner.

pub mod isolated;
pub mod native;

pub use isolated::{IsolatedExtensionLoader, IsolatedLoaderConfig, LoadedExtension};
pub use native::NativeExtensionMetadataLoader;
