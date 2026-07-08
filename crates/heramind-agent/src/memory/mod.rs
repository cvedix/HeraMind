//! Memory module for the HeraMind agent.
//!
//! Provides markdown-based memory with LLM extraction, dedup, compression,
//! and a scheduler for background maintenance tasks.

pub mod compressor;
pub mod dedup;
pub mod scheduler;
pub mod snapshot;

// Re-exports consumed via shortcut path (crate::memory::TypeName)
pub use scheduler::MemoryScheduler;
pub use snapshot::MemorySnapshot;
