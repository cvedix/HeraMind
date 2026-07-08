//! Dynamic library validation for native extensions
//!
//! This module provides pre-load validation of dynamic libraries to catch
//! common issues that would cause crashes during loading.
//!
//! # Supported Platforms
//!
//! - **macOS**: Validates LC_ID_DYLIB header (must be @rpath/extension.dylib)
//! - **Linux/Windows**: Basic file existence and format checks

use std::path::Path;
use tracing::info;
#[cfg(target_os = "macos")]
use tracing::warn;

/// Validation error types
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File is too small to be a valid library: {0} bytes")]
    FileTooSmall(u64),

    #[error("Invalid library format: {0}")]
    InvalidFormat(String),

    #[cfg(target_os = "macos")]
    #[error("macOS LC_ID_DYLIB validation failed: {0}")]
    InvalidDylibId(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Validate a dynamic library before loading
///
/// This function performs platform-specific validation to catch common issues
/// that would cause crashes during `dlopen()`/`LoadLibrary()`.
///
/// # Arguments
///
/// * `path` - Path to the dynamic library
///
/// # Returns
///
/// * `Ok(())` - Library is valid and safe to load
/// * `Err(ValidationError)` - Library has issues that would cause crashes
pub fn validate_library(path: &Path) -> Result<(), ValidationError> {
    // Check file exists
    if !path.exists() {
        return Err(ValidationError::FileNotFound(path.display().to_string()));
    }

    // Check minimum file size (a valid dylib/dll/so is at least 4KB)
    let metadata = std::fs::metadata(path)?;
    const MIN_SIZE: u64 = 4096;
    if metadata.len() < MIN_SIZE {
        return Err(ValidationError::FileTooSmall(metadata.len()));
    }

    // Platform-specific validation
    #[cfg(target_os = "macos")]
    {
        validate_macos_dylib(path)?;
    }

    #[cfg(target_os = "linux")]
    {
        validate_linux_so(path)?;
    }

    #[cfg(target_os = "windows")]
    {
        validate_windows_dll(path)?;
    }

    Ok(())
}

// ============================================================================
// macOS Validation
// ============================================================================

#[cfg(target_os = "macos")]
fn validate_macos_dylib(path: &Path) -> Result<(), ValidationError> {
    use std::process::Command;

    // Use otool to get LC_ID_DYLIB
    let output = Command::new("otool")
        .arg("-L")
        .arg(path)
        .output()
        .map_err(|e| {
            ValidationError::IoError(std::io::Error::other(format!("Failed to run otool: {}", e)))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ValidationError::InvalidFormat(format!(
            "otool failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // First line after header is the LC_ID_DYLIB (the library's own identity)
    // Format: "\tlibname (compatibility version, current version)"
    // or just: "path/to/lib (architecture)"
    if lines.len() < 2 {
        return Err(ValidationError::InvalidFormat(
            "Could not parse otool output".to_string(),
        ));
    }

    // Find the LC_ID_DYLIB entry
    // It's usually the first entry after the header, and should be the library's own identity
    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();

        // Skip empty lines and the header
        if line.is_empty() || i == 0 {
            continue;
        }

        // The first non-empty line after header should be the LC_ID_DYLIB
        // For a valid extension, it should be "@rpath/extension.dylib"
        // NOT an absolute build path like "/Users/.../extension.dylib"

        // Extract the path (before the version info in parentheses)
        let lib_path = line.split('(').next().map(|s| s.trim()).unwrap_or(line);

        // Check if this is an absolute path that shouldn't be there
        if lib_path.starts_with('/')
            && !lib_path.starts_with("/usr/lib")
            && !lib_path.starts_with("/System")
        {
            // This is likely the LC_ID_DYLIB with an absolute build path
            // This will cause crashes on other machines!
            if !lib_path.starts_with("@") {
                warn!(
                    path = %path.display(),
                    lc_id = %lib_path,
                    "LC_ID_DYLIB contains absolute build path! This extension will crash on other machines."
                );
                return Err(ValidationError::InvalidDylibId(format!(
                    "LC_ID_DYLIB '{}' is an absolute build path. It must be '@rpath/extension.dylib'. \
                     Fix with: install_name_tool -id '@rpath/extension.dylib' {}",
                    lib_path,
                    path.display()
                )));
            }
        }

        // If it starts with @rpath, that's correct
        if lib_path.starts_with("@rpath/") {
            info!(
                path = %path.display(),
                lc_id = %lib_path,
                "LC_ID_DYLIB is correctly configured"
            );
        }

        // Only check the first non-header line
        break;
    }

    Ok(())
}

// ============================================================================
// Linux Validation
// ============================================================================

#[cfg(target_os = "linux")]
fn validate_linux_so(path: &Path) -> Result<(), ValidationError> {
    use std::fs::File;
    use std::io::Read;

    // Check ELF magic number
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46]; // "\x7fELF"
    if magic != ELF_MAGIC {
        return Err(ValidationError::InvalidFormat(
            "Not a valid ELF file (wrong magic number)".to_string(),
        ));
    }

    info!(
        path = %path.display(),
        "ELF file validated successfully"
    );

    Ok(())
}

// ============================================================================
// Windows Validation
// ============================================================================

#[cfg(target_os = "windows")]
fn validate_windows_dll(path: &Path) -> Result<(), ValidationError> {
    use std::fs::File;
    use std::io::Read;

    // Check PE/DOS magic number
    let mut file = File::open(path)?;
    let mut magic = [0u8; 2];
    file.read_exact(&mut magic)?;

    const DOS_MAGIC: [u8; 2] = [0x4d, 0x5a]; // "MZ"
    if magic != DOS_MAGIC {
        return Err(ValidationError::InvalidFormat(
            "Not a valid PE/DLL file (wrong magic number)".to_string(),
        ));
    }

    info!(
        path = %path.display(),
        "PE/DLL file validated successfully"
    );

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::FileNotFound("test.dylib".to_string());
        assert!(err.to_string().contains("test.dylib"));

        #[cfg(target_os = "macos")]
        {
            let err = ValidationError::InvalidDylibId("bad id".to_string());
            assert!(err.to_string().contains("bad id"));
        }
    }

    #[test]
    fn test_validate_library_file_not_found() {
        let path = PathBuf::from("/nonexistent/path/test.dylib");
        let result = validate_library(&path);
        assert!(result.is_err());
        match result {
            Err(ValidationError::FileNotFound(msg)) => {
                assert!(msg.contains("nonexistent"));
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_validate_library_file_too_small() {
        let temp_dir = TempDir::new().unwrap();
        let small_file = temp_dir.path().join("tiny.dylib");

        // Create a file smaller than MIN_SIZE (4KB)
        let mut file = File::create(&small_file).unwrap();
        file.write_all(b"tiny").unwrap();
        file.sync_all().unwrap();

        let result = validate_library(&small_file);
        assert!(result.is_err());
        match result {
            Err(ValidationError::FileTooSmall(size)) => {
                assert_eq!(size, 4);
            }
            _ => panic!("Expected FileTooSmall error"),
        }
    }

    #[test]
    fn test_validate_library_exactly_min_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("exact.dylib");

        // Create a file exactly MIN_SIZE bytes
        let mut file = File::create(&file_path).unwrap();
        let data = vec![0u8; 4096];
        file.write_all(&data).unwrap();
        file.sync_all().unwrap();

        let result = validate_library(&file_path);
        // Should fail at format validation (not a real library)
        // but should NOT fail at size check
        #[cfg(target_os = "linux")]
        match result {
            Err(ValidationError::InvalidFormat(_)) => {
                // Expected - not a real ELF file
            }
            _ => {
                panic!("Expected InvalidFormat error for fake library");
            }
        }

        #[cfg(target_os = "macos")]
        match result {
            Err(ValidationError::InvalidFormat(_)) | Err(ValidationError::IoError(_)) => {
                // Expected - not a real dylib
            }
            _ => {
                panic!("Expected validation error for fake library");
            }
        }

        #[cfg(target_os = "windows")]
        match result {
            Err(ValidationError::InvalidFormat(_)) => {
                // Expected - not a real DLL
            }
            _ => {
                panic!("Expected validation error for fake library");
            }
        }
    }

    #[test]
    fn test_validate_library_zero_size() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file = temp_dir.path().join("empty.dylib");

        // Create an empty file
        File::create(&empty_file).unwrap().sync_all().unwrap();

        let result = validate_library(&empty_file);
        assert!(result.is_err());
        match result {
            Err(ValidationError::FileTooSmall(size)) => {
                assert_eq!(size, 0);
            }
            _ => panic!("Expected FileTooSmall error for empty file"),
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_validate_linux_so_magic_number() {
        let temp_dir = TempDir::new().unwrap();
        let fake_elf = temp_dir.path().join("fake.so");

        // Create a file with wrong magic number
        let mut file = File::create(&fake_elf).unwrap();
        let data = vec![0u8; 8192]; // Large enough to pass size check
        file.write_all(&data).unwrap();
        file.sync_all().unwrap();

        let result = validate_library(&fake_elf);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidFormat(msg)) => {
                assert!(msg.contains("ELF") || msg.contains("magic"));
            }
            _ => panic!("Expected InvalidFormat error for bad magic number"),
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_validate_linux_so_correct_magic() {
        let temp_dir = TempDir::new().unwrap();
        let elf_file = temp_dir.path().join("elf.so");

        // Create a file with correct ELF magic but otherwise invalid
        let mut file = File::create(&elf_file).unwrap();
        let magic: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46]; // ELF magic
        file.write_all(&magic).unwrap();
        let padding = vec![0u8; 8192 - 4];
        file.write_all(&padding).unwrap();
        file.sync_all().unwrap();

        let result = validate_library(&elf_file);
        // Should pass magic number check but will fail later validation
        // or succeed if we only check magic (which we do)
        // The current implementation only checks magic, so it should succeed
        assert!(result.is_ok());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_validate_windows_dll_magic_number() {
        let temp_dir = TempDir::new().unwrap();
        let fake_dll = temp_dir.path().join("fake.dll");

        // Create a file with wrong magic number
        let mut file = File::create(&fake_dll).unwrap();
        let data = vec![0u8; 8192]; // Large enough
        file.write_all(&data).unwrap();
        file.sync_all().unwrap();

        let result = validate_library(&fake_dll);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidFormat(msg)) => {
                assert!(msg.contains("PE") || msg.contains("magic"));
            }
            _ => panic!("Expected InvalidFormat error for bad magic number"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_validate_windows_dll_correct_magic() {
        let temp_dir = TempDir::new().unwrap();
        let dll_file = temp_dir.path().join("dll.dll");

        // Create a file with correct DOS magic (MZ)
        let mut file = File::create(&dll_file).unwrap();
        let magic: [u8; 2] = [0x4d, 0x5a]; // "MZ"
        file.write_all(&magic).unwrap();
        let padding = vec![0u8; 8192 - 2];
        file.write_all(&padding).unwrap();
        file.sync_all().unwrap();

        let result = validate_library(&dll_file);
        // Should pass magic number check
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let validation_err: ValidationError = io_err.into();
        match validation_err {
            ValidationError::IoError(_) => {
                // Expected
            }
            _ => panic!("Expected IoError variant"),
        }
    }
}
