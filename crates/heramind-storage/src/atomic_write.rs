//! Atomic file write helper.
//!
//! Provides `write()` which writes content to a sibling temp file then
//! `rename(2)`s over the target. POSIX `rename` and Windows
//! `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` both guarantee the
//! destination ends up as either the complete old version or the
//! complete new version — never a half-written intermediate state.
//!
//! This is critical for files like `USER.md` / `KNOWLEDGE.md` where a
//! partial write would corrupt all agent memory and be unrecoverable.
//! The historical pattern of `fs::write(path, content)` does
//! `open(O_TRUNC) → write → close`, which leaves a truncated file if
//! the process is killed (panic, OOM, SIGKILL, power loss) mid-write.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Atomically write `content` to `path`.
///
/// Writes a sibling hidden temp file `<filename>.tmp` (in the same
/// directory — required so `rename` doesn't degrade to non-atomic
/// copy+unlink across filesystems), then renames over the target.
/// Cleans up the temp file on either step failing.
///
/// Accepts `impl AsRef<[u8]>` so callers can pass `&str` or `&[u8]`
/// without manual conversion — drop-in replacement for `fs::write`.
///
/// # Errors
///
/// Returns `io::Error` for any underlying filesystem failure. The
/// caller is responsible for mapping to its own error type.
pub fn write(path: &Path, content: impl AsRef<[u8]>) -> io::Result<()> {
    let tmp = tmp_path_for(path);

    if let Err(e) = fs::write(&tmp, content.as_ref()) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }

    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }

    Ok(())
}

/// Build the temp-file path used by `write`.
///
/// Format: `<dirname>/.<basename>.tmp` (hidden file in the same dir).
fn tmp_path_for(path: &Path) -> PathBuf {
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp_name = format!(".{}.tmp", filename);
    path.with_file_name(tmp_name)
}

/// Remove leftover `.<name>.tmp` files in `dir` produced by aborted atomic
/// writes (process killed between `fs::write(tmp)` and `fs::rename(tmp, path)`).
///
/// Best-effort: errors reading individual directory entries or removing
/// individual files are logged at `debug!` and skipped, so a permission
/// issue on one file doesn't prevent startup. Returns the number of files
/// removed. Safe to call on a non-existent directory (returns 0).
///
/// Call this from store `init()` paths so a long-running server doesn't
/// accumulate hidden tmp files across crashes.
pub fn cleanup_stale_temps(dir: &Path) -> usize {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let is_tmp = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|name| name.starts_with('.') && name.ends_with(".tmp"))
            .unwrap_or(false);
        if !is_tmp {
            continue;
        }
        // Best-effort removal — never propagate errors. A leftover tmp from
        // a concurrent write-in-progress would be caught here too, but
        // `remove_file` on a file that's about to be renamed is harmless on
        // both POSIX and Windows (the rename either already happened → file
        // gone → no-op; or hasn't happened → we win the race and the
        // subsequent rename fails, which the writer handles via cleanup).
        if fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fresh.md");
        write(&path, b"hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn atomic_write_replaces_existing_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("USER.md");
        fs::write(&path, "old").unwrap();

        write(&path, b"new content").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new content");
    }

    #[test]
    fn atomic_write_leaves_no_temp_after_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.md");
        write(&path, b"data").unwrap();

        let tmp = tmp_path_for(&path);
        assert!(!tmp.exists(), "temp file should be gone after success");
    }

    #[test]
    fn tmp_path_is_sibling_in_same_dir() {
        let path = Path::new("/tmp/agent/USER.md");
        let tmp = tmp_path_for(path);
        assert_eq!(tmp, Path::new("/tmp/agent/.USER.md.tmp"));
        assert_eq!(tmp.parent(), path.parent(), "temp must share parent dir");
    }

    #[test]
    fn cleanup_removes_stale_temps_only() {
        let dir = tempfile::tempdir().unwrap();
        // Simulate leftovers from an aborted write
        fs::write(dir.path().join(".USER.md.tmp"), "partial").unwrap();
        fs::write(dir.path().join(".KNOWLEDGE.md.tmp"), "partial").unwrap();
        // Real files and unrelated files must be preserved
        fs::write(dir.path().join("USER.md"), "real").unwrap();
        fs::write(dir.path().join("notes.txt"), "keep me").unwrap();
        // A `.tmp` extension without leading `.` should NOT match (named like a
        // user-created file, not an atomic-write temp)
        fs::write(dir.path().join("data.tmp"), "keep").unwrap();

        let removed = cleanup_stale_temps(dir.path());

        assert_eq!(removed, 2, "only the two hidden .*.tmp files should be removed");
        assert!(!dir.path().join(".USER.md.tmp").exists());
        assert!(!dir.path().join(".KNOWLEDGE.md.tmp").exists());
        assert!(dir.path().join("USER.md").exists(), "real file preserved");
        assert!(dir.path().join("notes.txt").exists(), "unrelated file preserved");
        assert!(dir.path().join("data.tmp").exists(), "non-hidden .tmp file preserved");
    }

    #[test]
    fn cleanup_on_missing_dir_is_noop() {
        let removed = cleanup_stale_temps(Path::new("/no/such/dir/anywhere"));
        assert_eq!(removed, 0);
    }
}
