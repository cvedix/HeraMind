//! Cryptographic utilities for API key encryption.
//!
//! The implementation lives in `heramind_core::crypto` so every layer (api,
//! storage, …) shares the same `data/encryption_key`. This module re-exports
//! it to keep the historical `crate::crypto::…` paths working.

pub use heramind_core::crypto::{CryptoError, CryptoService};
