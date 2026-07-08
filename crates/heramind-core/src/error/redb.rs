//! Centralized redb error conversions
//!
//! All crates should use these conversions to avoid duplicating `From` implementations.
//! This module provides consistent error handling for the redb embedded database.

use super::Error;

impl From<redb::Error> for Error {
    fn from(e: redb::Error) -> Self {
        Error::Storage(format!("Redb error: {}", e))
    }
}

impl From<redb::StorageError> for Error {
    fn from(e: redb::StorageError) -> Self {
        Error::Storage(format!("Redb storage error: {}", e))
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(e: redb::DatabaseError) -> Self {
        Error::Storage(format!("Redb database error: {}", e))
    }
}

impl From<redb::TableError> for Error {
    fn from(e: redb::TableError) -> Self {
        Error::Storage(format!("Redb table error: {}", e))
    }
}

impl From<redb::CommitError> for Error {
    fn from(e: redb::CommitError) -> Self {
        Error::Storage(format!("Redb commit error: {}", e))
    }
}

impl From<redb::TransactionError> for Error {
    fn from(e: redb::TransactionError) -> Self {
        Error::Storage(format!("Redb transaction error: {}", e))
    }
}

impl From<redb::CompactionError> for Error {
    fn from(e: redb::CompactionError) -> Self {
        Error::Storage(format!("Redb compaction error: {}", e))
    }
}
