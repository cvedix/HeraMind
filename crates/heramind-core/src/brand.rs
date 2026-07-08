//! Brand configuration for the application.
//!
//! This module provides centralized brand information (name, version, etc.)
//! to make it easy to rebrand the application.
//!
//! ## Rebranding
//! To rebrand the application, modify the constants in this module.
//!
//! ## Example
//! ```rust
//! use heramind_core::brand::APP_NAME;
//!
//! println!("Welcome to {}", APP_NAME);
//! ```

/// Full application name
pub const APP_NAME: &str = "HeraMind";

/// Application version (from Cargo.toml)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default user agent string for HTTP requests
pub fn user_agent() -> String {
    format!("{}/{}", APP_NAME, APP_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_name() {
        assert_eq!(APP_NAME, "HeraMind");
    }

    #[test]
    fn test_user_agent() {
        let ua = user_agent();
        assert!(ua.contains("HeraMind"));
        assert!(ua.contains('.'));
    }
}
