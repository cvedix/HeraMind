//! Resource registry for semantic mapping.
//!
//! Provides resource indexing for natural-language → technical ID resolution
//! (e.g., "客厅灯" → device_id). Used by `SemanticMapper` in the agent crate.

mod resource_index;

pub use resource_index::{Resource, ResourceDataHelper, ResourceIndex};
