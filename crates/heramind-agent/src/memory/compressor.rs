//! Memory compressor — simplified to char-based eviction.
//!
//! Removes oldest entries (from the bottom of the file) when the file
//! exceeds the character limit. No LLM calls needed.

/// Result of an eviction operation.
#[derive(Debug, Clone)]
pub struct EvictionResult {
    /// Content after eviction (may be unchanged if under limit)
    pub content: String,
    /// Number of lines removed
    pub lines_removed: usize,
    /// Whether eviction was performed
    pub evicted: bool,
}

/// Evict content to fit within max_chars by removing lines from the bottom.
///
/// This is a simple, deterministic approach — no LLM needed.
/// Lines are removed from the bottom of the file.
///
/// `max_chars` is a **character** count, not bytes — important for multi-byte
/// UTF-8 content (e.g. Chinese, where each char is 3 bytes). Using byte length
/// here would evict at ~1/3 the intended budget for Chinese text.
pub fn evict_to_limit(content: &str, max_chars: usize) -> EvictionResult {
    let char_count = content.chars().count();
    if char_count <= max_chars {
        return EvictionResult {
            content: content.to_string(),
            lines_removed: 0,
            evicted: false,
        };
    }

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Pre-compute cumulative CHARACTER counts from the start.
    // prefix_chars[i] = chars in lines[0..i] including newlines between them.
    let mut prefix_chars = vec![0usize; total_lines + 1];
    for i in 0..total_lines {
        let newline = if i > 0 { 1 } else { 0 };
        prefix_chars[i + 1] = prefix_chars[i] + lines[i].chars().count() + newline;
    }

    // Binary search for the largest keep_count where prefix_chars[keep_count] <= max_chars
    let mut lo = 0usize;
    let mut hi = total_lines;
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if prefix_chars[mid] <= max_chars {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    let keep_count = lo;
    let removed = total_lines - keep_count;

    EvictionResult {
        content: if keep_count > 0 {
            lines[..keep_count].join("\n")
        } else {
            String::new()
        },
        lines_removed: removed,
        evicted: removed > 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_under_limit() {
        let content = "hello world";
        let result = evict_to_limit(content, 100);
        assert!(!result.evicted);
        assert_eq!(result.content, content);
    }

    #[test]
    fn test_over_limit_removes_lines() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        let result = evict_to_limit(content, 20);
        assert!(result.evicted);
        assert!(result.lines_removed > 0);
        assert!(result.content.chars().count() <= 20);
    }

    #[test]
    fn test_empty_content() {
        let result = evict_to_limit("", 100);
        assert!(!result.evicted);
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_exact_limit() {
        let content = "hello";
        let result = evict_to_limit(content, 5);
        assert!(!result.evicted);
        assert_eq!(result.content, "hello");
    }

    #[test]
    fn test_one_char_over_limit() {
        let content = "hello world";
        let result = evict_to_limit(content, 10);
        assert!(result.evicted);
        assert!(result.content.chars().count() <= 10);
    }

    #[test]
    fn test_multiline_with_newlines() {
        let content = "line 1\nline 2\nline 3";
        let total_with_newlines = content.chars().count() + 2; // 2 newlines
        let result = evict_to_limit(content, total_with_newlines);
        assert!(!result.evicted);
    }

    #[test]
    fn test_eviction_preserves_order() {
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        let result = evict_to_limit(content, 15);
        assert!(result.evicted);
        // Should keep first lines, remove from bottom
        assert!(result.content.starts_with("line 1"));
        assert!(result.content.contains("line 2"));
    }

    /// Regression: byte vs char mismatch previously evicted Chinese at ~1/3
    /// the intended budget (3 bytes/char). Verify a Chinese line budget is
    /// measured in characters, not bytes.
    #[test]
    fn test_eviction_uses_char_count_for_multibyte() {
        // 3 lines × 10 Chinese chars = 30 chars total (90 bytes UTF-8).
        // Newlines add 2 more chars → 32 chars total.
        let line = "一二三四五六七八九十"; // 10 chars
        let content = format!("{}\n{}\n{}", line, line, line);

        // Budget = 21 chars → should keep first 2 lines (10 + 1 newline + 10 = 21).
        let result = evict_to_limit(&content, 21);
        assert!(result.evicted, "should evict at char limit");
        assert!(
            result.content.chars().count() <= 21,
            "kept content must be within char budget, got {} chars",
            result.content.chars().count()
        );
        assert_eq!(result.lines_removed, 1, "should drop exactly the 3rd line");
        // First two lines preserved
        assert_eq!(result.content, format!("{}\n{}", line, line));
    }
}
