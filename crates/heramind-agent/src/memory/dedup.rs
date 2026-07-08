//! Memory deduplication module
//!
//! Detects similar memory entries using character n-gram Jaccard similarity.
//! Uses bigrams (2-character sequences) to properly handle Chinese and other
//! languages that don't use spaces between words.

use std::collections::HashSet;

/// Default n-gram size (bigrams)
const DEFAULT_NGRAM_SIZE: usize = 2;

/// Deduplication processor using character n-gram Jaccard similarity
#[derive(Debug, Clone)]
pub struct DedupProcessor {
    similarity_threshold: f32,
    ngram_size: usize,
}

impl DedupProcessor {
    /// Create a new dedup processor
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            similarity_threshold,
            ngram_size: DEFAULT_NGRAM_SIZE,
        }
    }

    /// Create with default threshold (0.85)
    pub fn with_defaults() -> Self {
        Self::new(0.85)
    }

    /// Generate character n-grams from text
    fn char_ngrams(s: &str, n: usize) -> HashSet<String> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() < n {
            // For very short strings, use the whole string as one n-gram
            return HashSet::from_iter([s.to_string()]);
        }
        (0..=chars.len() - n)
            .map(|i| chars[i..i + n].iter().collect())
            .collect()
    }

    /// Calculate Jaccard similarity with configurable n-gram size
    fn jaccard_similarity_with_ngram(a: &str, b: &str, n: usize) -> f32 {
        let set_a = Self::char_ngrams(a.trim(), n);
        let set_b = Self::char_ngrams(b.trim(), n);

        if set_a.is_empty() && set_b.is_empty() {
            return 1.0;
        }

        if set_a.is_empty() || set_b.is_empty() {
            return 0.0;
        }

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        intersection as f32 / union as f32
    }

    /// Compute raw Jaccard similarity between two strings (0.0–1.0).
    /// Public so callers can do block-vs-block comparisons directly.
    pub fn similarity(&self, a: &str, b: &str) -> f32 {
        Self::jaccard_similarity_with_ngram(a, b, self.ngram_size)
    }

    /// Find similar entry in existing list
    pub fn find_similar(&self, content: &str, existing: &[String]) -> Option<(usize, f32)> {
        for (i, entry) in existing.iter().enumerate() {
            let similarity = Self::jaccard_similarity_with_ngram(content, entry, self.ngram_size);
            if similarity >= self.similarity_threshold {
                return Some((i, similarity));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_similar_chinese() {
        let dedup = DedupProcessor::new(0.6);
        let existing = vec!["温度传感器读数为25度".to_string()];

        let result = dedup.find_similar("温度传感器读数为26度", &existing);
        assert!(result.is_some(), "Should detect similar Chinese entries");
        let (_, sim) = result.unwrap();
        assert!(sim >= 0.6, "sim was {}", sim);
    }

    #[test]
    fn test_find_similar_not_found() {
        let dedup = DedupProcessor::new(0.85);
        let existing = vec!["温度监控正常".to_string()];

        let result = dedup.find_similar("用户偏好中文", &existing);
        assert!(result.is_none());
    }
}
