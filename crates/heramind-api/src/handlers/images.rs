//! Static image file server for data/images/.
//!
//! Serves files produced by image_edit and similar tools via
//! GET /api/images/<filename>. Used by chat markdown rendering:
//! the LLM writes `![alt](/api/images/foo.png)` and the browser
//! fetches the image from this route.

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::PathBuf;

use crate::handlers::ServerState;

/// Allowed extensions (lowercase, no leading dot).
const ALLOWED_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff"];

/// Cache for 30 days. Files are content-addressed (UUID-based names) so the
/// same filename never refers to different bytes.
#[allow(dead_code)]
const CACHE_MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60;

pub async fn get_image_handler(
    State(state): State<ServerState>,
    Path(filename): Path<String>,
) -> Response {
    // 1. Validate filename — reject anything that smells like path traversal.
    if !is_safe_filename(&filename) {
        return (StatusCode::BAD_REQUEST, "invalid filename").into_response();
    }

    let images_dir = state.data_dir.join("images");
    let file_path: PathBuf = images_dir.join(&filename);

    // 2. Resolve canonical paths and verify the file is actually inside
    //    images_dir. This defeats symlinks: if `images_dir/foo.png` is a
    //    symlink to `/etc/passwd`, canonicalize() resolves it and the
    //    starts_with check fails.
    let canon_images = match images_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "images dir not present").into_response(),
    };
    let canon_file = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "image not found").into_response(),
    };
    if !canon_file.starts_with(&canon_images) {
        // Symlink escape attempt.
        tracing::warn!(
            filename = %filename,
            "rejected image path outside images dir"
        );
        return (StatusCode::NOT_FOUND, "image not found").into_response();
    }

    // 3. Read bytes (could use tokio fs, but images are small — sync read in
    //    blocking task is fine).
    let bytes = match tokio::task::spawn_blocking(move || std::fs::read(&file_path)).await {
        Ok(Ok(b)) => b,
        Ok(Err(_)) => return (StatusCode::NOT_FOUND, "image not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "read failed").into_response(),
    };

    // 4. Derive Content-Type from extension.
    let content_type = mime_from_ext(&filename);

    // 5. Build response with immutable cache headers.
    let mut resp = bytes.into_response();
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=2592000, immutable"),
    );
    resp
}

/// Validate that `name` is a safe single-segment filename.
fn is_safe_filename(name: &str) -> bool {
    if name.is_empty() || name.starts_with('.') {
        return false;
    }
    // Reject any path separator or traversal token.
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return false;
    }
    if name.contains('\0') {
        return false;
    }
    // Must have an allowed extension.
    let lower = name.to_ascii_lowercase();
    let Some(ext) = lower.rsplit('.').next() else {
        return false;
    };
    if !ALLOWED_EXTS.contains(&ext) {
        return false;
    }
    // All chars must be in safe set: alphanumeric, dash, underscore, dot.
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return false;
    }
    true
}

/// Map a filename extension to a MIME type. Falls back to JPEG.
fn mime_from_ext(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tiff" => "image/tiff",
        _ => "image/jpeg",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_filenames_accepted() {
        assert!(is_safe_filename("abc123.png"));
        assert!(is_safe_filename(
            "550e8400-e29b-41d4-a716-446655440000.jpeg"
        ));
        assert!(is_safe_filename("image_edit_2026.webp"));
    }

    #[test]
    fn path_traversal_rejected() {
        assert!(!is_safe_filename("../etc/passwd.png"));
        assert!(!is_safe_filename("a/b.png"));
        assert!(!is_safe_filename("a\\b.png"));
        assert!(!is_safe_filename(".hidden.png"));
        assert!(!is_safe_filename("foo..png")); // contains ".."
        assert!(!is_safe_filename("foo\x00.png")); // null byte
    }

    #[test]
    fn bad_extensions_rejected() {
        assert!(!is_safe_filename("foo.txt"));
        assert!(!is_safe_filename("foo")); // no extension
        assert!(!is_safe_filename("foo.exe"));
        assert!(!is_safe_filename("foo.PnG.exe"));
    }

    #[test]
    fn unsafe_chars_rejected() {
        assert!(!is_safe_filename("foo bar.png")); // space
        assert!(!is_safe_filename("foo+bar.png")); // plus
        assert!(!is_safe_filename("café.png")); // non-ascii
    }

    #[test]
    fn mime_mapping() {
        assert_eq!(mime_from_ext("x.png"), "image/png");
        assert_eq!(mime_from_ext("x.JPG"), "image/jpeg");
        assert_eq!(mime_from_ext("x.webp"), "image/webp");
        assert_eq!(mime_from_ext("x.unknown"), "image/jpeg");
    }
}
