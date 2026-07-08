//! Local IPC extensions for isolated extensions
//!
//! This module re-exports IPC types from the SDK and may contain
//! additional local implementations specific to the main process.

// All IPC protocol types are now defined in heramind-extension-sdk
// and re-exported via super::super::system module.
// This file exists for potential local extensions only.

// Re-exports are handled by mod.rs directly
// This file is kept for future local extensions if needed
