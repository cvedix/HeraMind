//! Rule persistence using redb database.
//!
//! Provides persistent storage for rule definitions and execution history.

use crate::models::{CompiledRule, RuleId};
use parking_lot::Mutex;
use redb::{Database, ReadableTable, TableDefinition};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Table definitions
const RULES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("rules");
const HISTORY_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("rule_history");

/// Error type for rule storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Implement From for all redb error types
impl From<redb::Error> for StoreError {
    fn from(e: redb::Error) -> Self {
        StoreError::Database(e.to_string())
    }
}

impl From<redb::StorageError> for StoreError {
    fn from(e: redb::StorageError) -> Self {
        StoreError::Database(e.to_string())
    }
}

impl From<redb::DatabaseError> for StoreError {
    fn from(e: redb::DatabaseError) -> Self {
        StoreError::Database(e.to_string())
    }
}

impl From<redb::TableError> for StoreError {
    fn from(e: redb::TableError) -> Self {
        StoreError::Database(e.to_string())
    }
}

impl From<redb::TransactionError> for StoreError {
    fn from(e: redb::TransactionError) -> Self {
        StoreError::Database(e.to_string())
    }
}

impl From<redb::CommitError> for StoreError {
    fn from(e: redb::CommitError) -> Self {
        StoreError::Database(e.to_string())
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Serialization(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// Persistent storage for rules.
pub struct RuleStore {
    db: Arc<Database>,
    /// Storage path for singleton tracking.
    path: String,
    /// Temp file path for cleanup (if using memory mode).
    temp_path: Option<PathBuf>,
}

/// Global rule store singleton to prevent multiple opens.
static RULE_STORE_SINGLETON: Mutex<Option<Arc<RuleStore>>> = Mutex::new(None);

impl RuleStore {
    /// Open or create a rule store at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Arc<Self>> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Check singleton
        {
            let singleton = RULE_STORE_SINGLETON.lock();
            if let Some(store) = singleton.as_ref() {
                if store.path == path_str {
                    return Ok(store.clone());
                }
            }
        }

        // Open database
        let (db, temp_path) = Self::open_db(&path_str)?;

        let store = Arc::new(RuleStore {
            db: Arc::new(db),
            path: path_str,
            temp_path,
        });

        *RULE_STORE_SINGLETON.lock() = Some(store.clone());
        Ok(store)
    }

    /// Create an in-memory store.
    pub fn memory() -> Result<Arc<Self>> {
        let temp_path =
            std::env::temp_dir().join(format!("rules_store_{}.redb", uuid::Uuid::new_v4()));
        Self::open(temp_path)
    }

    fn open_db(path_str: &str) -> Result<(Database, Option<PathBuf>)> {
        let (db, temp_path) = if path_str == ":memory:" {
            // Use temp file for in-memory mode
            let temp_path =
                std::env::temp_dir().join(format!("rules_store_{}.redb", uuid::Uuid::new_v4()));
            let db = Database::create(&temp_path)?;
            (db, Some(temp_path))
        } else {
            let path_ref = Path::new(path_str);
            if let Some(parent) = path_ref.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let db = if path_ref.exists() {
                Database::open(path_ref)?
            } else {
                Database::create(path_ref)?
            };
            (db, None)
        };

        Ok((db, temp_path))
    }

    /// Save a rule.
    pub fn save(&self, rule: &CompiledRule) -> Result<()> {
        let key = format!("rule:{}", rule.id);
        let value = serde_json::to_vec(rule)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(RULES_TABLE)?;
            table.insert(key.as_str(), value.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a rule by ID.
    pub fn load(&self, id: &RuleId) -> Result<Option<CompiledRule>> {
        let key = format!("rule:{}", id);

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(RULES_TABLE)?;

        match table.get(key.as_str())? {
            Some(value) => {
                let rule = serde_json::from_slice(value.value())?;
                Ok(Some(rule))
            }
            None => Ok(None),
        }
    }

    /// Delete a rule by ID.
    pub fn delete(&self, id: &RuleId) -> Result<bool> {
        let key = format!("rule:{}", id);

        let write_txn = self.db.begin_write()?;
        let existed = {
            let mut table = write_txn.open_table(RULES_TABLE)?;
            let result = table.remove(key.as_str())?.is_some();
            result
        };
        write_txn.commit()?;
        Ok(existed)
    }

    /// List all rules.
    pub fn list_all(&self) -> Result<Vec<CompiledRule>> {
        let mut rules = Vec::new();

        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(RULES_TABLE) {
            Ok(t) => t,
            Err(_) => return Ok(rules), // Table doesn't exist yet
        };

        let iter = table.iter()?;
        for result in iter {
            let (_, value) = result?;
            let rule: CompiledRule = serde_json::from_slice(value.value())?;
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Save an execution result to history.
    pub fn save_history(&self, result: &crate::models::RuleExecutionResult) -> Result<()> {
        // Key: timestamp + rule_id for ordering
        let key = format!(
            "history:{}:{}",
            result.triggered_at.timestamp_millis(),
            result.rule_id
        );
        let value = serde_json::to_vec(result)?;

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(HISTORY_TABLE)?;
            table.insert(key.as_str(), value.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load execution history for a specific rule.
    pub fn load_history(
        &self,
        rule_id: &RuleId,
    ) -> Result<Vec<crate::models::RuleExecutionResult>> {
        let mut results = Vec::new();

        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(HISTORY_TABLE) {
            Ok(t) => t,
            Err(_) => return Ok(results), // Table doesn't exist yet
        };

        let iter = table.iter()?;
        for item in iter {
            let (_, value) = item?;
            let entry: crate::models::RuleExecutionResult = serde_json::from_slice(value.value())?;
            if &entry.rule_id == rule_id {
                results.push(entry);
            }
        }

        // Sort by triggered_at descending (most recent first)
        results.sort_by(|a, b| b.triggered_at.cmp(&a.triggered_at));
        Ok(results)
    }

    /// Count history entries since a timestamp (only actual triggers with executed actions).
    pub fn count_history_since(&self, since_timestamp: i64) -> Result<u64> {
        let read_txn = self.db.begin_read()?;
        let table = match read_txn.open_table(HISTORY_TABLE) {
            Ok(t) => t,
            Err(_) => return Ok(0),
        };

        let mut count = 0u64;
        for result in table.iter()? {
            let (_, value) = result?;
            if let Ok(entry) =
                serde_json::from_slice::<crate::models::RuleExecutionResult>(value.value())
            {
                if entry.triggered_at.timestamp() >= since_timestamp
                    && !entry.actions_executed.is_empty()
                {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Clean up old history entries (older than the given number of days).
    pub fn cleanup_history(&self, older_than_days: u64) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);
        let cutoff_key = format!("history:{}:", cutoff.timestamp_millis());

        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(HISTORY_TABLE)?;
            let keys_to_remove: Vec<String> = table
                .iter()?
                .filter_map(|item| {
                    let (key, _) = item.ok()?;
                    let key_str = key.value().to_string();
                    if key_str < cutoff_key {
                        Some(key_str)
                    } else {
                        None
                    }
                })
                .collect();

            let count = keys_to_remove.len();
            for key in &keys_to_remove {
                table.remove(key.as_str())?;
            }
            count
        };
        write_txn.commit()?;
        Ok(removed)
    }
}

impl Drop for RuleStore {
    fn drop(&mut self) {
        // Clean up temp file if using memory mode
        if let Some(ref temp_path) = self.temp_path {
            let _ = std::fs::remove_file(temp_path);
        }
    }
}
