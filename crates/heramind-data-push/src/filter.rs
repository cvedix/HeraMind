//! Data source filtering logic.

use crate::types::DataSourceFilter;

/// Extended filtering with change detection support.
pub struct DataSourceMatcher {
    filter: DataSourceFilter,
    /// Last known values per source_id for change detection.
    last_values: std::collections::HashMap<String, String>,
}

impl DataSourceMatcher {
    pub fn new(filter: DataSourceFilter) -> Self {
        Self {
            filter,
            last_values: std::collections::HashMap::new(),
        }
    }

    /// Check if a data source matches and (if only_changes) has a new value.
    /// Returns true if the data should be pushed.
    pub fn should_push(&mut self, source_id: &str, value: &str) -> bool {
        if !self.filter.matches(source_id) {
            return false;
        }
        if self.filter.only_changes {
            if let Some(last) = self.last_values.get(source_id) {
                if last == value {
                    return false;
                }
            }
        }
        self.last_values
            .insert(source_id.to_string(), value.to_string());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_push_matches() {
        let filter = DataSourceFilter {
            source_patterns: vec!["device:s1:".to_string()],
            only_changes: false,
        };
        let mut matcher = DataSourceMatcher::new(filter);
        assert!(matcher.should_push("device:s1:temp", "25.0"));
        assert!(matcher.should_push("device:s1:temp", "25.0")); // always true without only_changes
    }

    #[test]
    fn test_should_push_only_changes() {
        let filter = DataSourceFilter {
            source_patterns: vec!["device:s1:".to_string()],
            only_changes: true,
        };
        let mut matcher = DataSourceMatcher::new(filter);
        assert!(matcher.should_push("device:s1:temp", "25.0")); // first time
        assert!(!matcher.should_push("device:s1:temp", "25.0")); // same value
        assert!(matcher.should_push("device:s1:temp", "26.0")); // changed
    }

    #[test]
    fn test_should_push_no_match() {
        let filter = DataSourceFilter {
            source_patterns: vec!["device:s1:".to_string()],
            only_changes: false,
        };
        let mut matcher = DataSourceMatcher::new(filter);
        assert!(!matcher.should_push("device:s2:temp", "25.0"));
    }
}
