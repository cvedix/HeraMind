use std::{fs, path::Path};

use anyhow::Result;
use chrono::Utc;
use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Job, JobStatus, Page};

pub const RETENTION_SECONDS: i64 = 24 * 60 * 60;
pub const MAX_JOBS: u64 = 200;
const JOBS: TableDefinition<&str, &str> = TableDefinition::new("ocr_jobs");
const RESULTS: TableDefinition<&str, &str> = TableDefinition::new("ocr_results");
const KEYS: TableDefinition<&str, &str> = TableDefinition::new("ocr_keys");

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Key {
    pub id: Uuid,
    pub name: String,
    pub created_at: i64,
}

pub fn open(directory: &Path) -> Result<Database> {
    fs::create_dir_all(directory)?;
    let db = Database::create(directory.join("ocr.redb"))?;
    let work = directory.join("work");
    fs::create_dir_all(&work)?;
    for entry in fs::read_dir(work)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.file_name().to_string_lossy().starts_with("job-") {
            fs::remove_dir_all(entry.path())?;
        }
    }

    let txn = db.begin_write()?;
    {
        let mut jobs = txn.open_table(JOBS)?;
        txn.open_table(RESULTS)?;
        txn.open_table(KEYS)?;
        let interrupted = jobs
            .iter()?
            .map(|row| -> Result<Job> { Ok(serde_json::from_str(row?.1.value())?) })
            .filter_map(|job| match job {
                Ok(job) if job.status.is_active() => Some(Ok(job)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<Vec<_>>>()?;
        for mut job in interrupted {
            job.fail("INTERRUPTED");
            jobs.insert(
                job.id.to_string().as_str(),
                serde_json::to_string(&job)?.as_str(),
            )?;
        }
    }
    txn.commit()?;
    Ok(db)
}

pub fn submit(db: &Database, job: &Job) -> Result<bool> {
    let txn = db.begin_write()?;
    {
        let mut jobs = txn.open_table(JOBS)?;
        let mut results = txn.open_table(RESULTS)?;
        let mut completed = Vec::new();
        let now = Utc::now().timestamp();
        for row in jobs.iter()? {
            let (_, value) = row?;
            let existing: Job = serde_json::from_str(value.value())?;
            if !existing.status.is_active() {
                completed.push(existing);
            }
        }
        completed.sort_by_key(|item| item.finished_at);
        for old in completed {
            if !old.expired(now) && jobs.len()? < MAX_JOBS {
                break;
            }
            jobs.remove(old.id.to_string().as_str())?;
            results.remove(old.id.to_string().as_str())?;
        }
        if jobs.len()? >= MAX_JOBS {
            return Ok(false);
        }
        jobs.insert(
            job.id.to_string().as_str(),
            serde_json::to_string(job)?.as_str(),
        )?;
    }
    txn.commit()?;
    Ok(true)
}

pub fn save(db: &Database, job: &Job, pages: Option<&[Page]>) -> Result<()> {
    let txn = db.begin_write()?;
    {
        let id = job.id.to_string();
        let mut jobs = txn.open_table(JOBS)?;
        let active = match jobs.get(id.as_str())? {
            Some(value) => serde_json::from_str::<Job>(value.value())?
                .status
                .is_active(),
            None => false,
        };
        if !active {
            return Ok(());
        }
        jobs.insert(id.as_str(), serde_json::to_string(job)?.as_str())?;
        if let Some(pages) = pages {
            txn.open_table(RESULTS)?
                .insert(id.as_str(), serde_json::to_string(pages)?.as_str())?;
        }
    }
    txn.commit()?;
    Ok(())
}

pub fn list(db: &Database, owner: Option<&str>) -> Result<Vec<Job>> {
    let txn = db.begin_read()?;
    let jobs = txn.open_table(JOBS)?;
    let now = Utc::now().timestamp();
    let mut list = Vec::new();
    for row in jobs.iter()? {
        let (_, value) = row?;
        let job: Job = serde_json::from_str(value.value())?;
        if !job.expired(now) && owner.is_none_or(|owner| owner == job.owner) {
            list.push(job);
        }
    }
    list.sort_by_key(|job| std::cmp::Reverse((job.created_at, job.id)));
    list.truncate(50);
    Ok(list)
}

pub fn get(db: &Database, id: Uuid, owner: Option<&str>) -> Result<Option<(Job, Vec<Page>)>> {
    let txn = db.begin_read()?;
    let jobs = txn.open_table(JOBS)?;
    let id = id.to_string();
    let Some(value) = jobs.get(id.as_str())? else {
        return Ok(None);
    };
    let job: Job = serde_json::from_str(value.value())?;
    if job.expired(Utc::now().timestamp()) || owner.is_some_and(|owner| owner != job.owner) {
        return Ok(None);
    }
    let pages = if job.status == JobStatus::Completed {
        let results = txn.open_table(RESULTS)?;
        let value = results
            .get(id.as_str())?
            .ok_or_else(|| anyhow::anyhow!("OCR result missing"))?;
        serde_json::from_str(value.value())?
    } else {
        Vec::new()
    };
    Ok(Some((job, pages)))
}

pub enum DeleteJobOutcome {
    Deleted,
    NotFound,
    Active,
}

pub fn delete_job(db: &Database, id: Uuid, owner: Option<&str>) -> Result<DeleteJobOutcome> {
    let txn = db.begin_write()?;
    {
        let id = id.to_string();
        let mut jobs = txn.open_table(JOBS)?;
        let job: Job = match jobs.get(id.as_str())? {
            Some(value) => serde_json::from_str(value.value())?,
            None => return Ok(DeleteJobOutcome::NotFound),
        };
        if job.expired(Utc::now().timestamp()) || owner.is_some_and(|owner| owner != job.owner) {
            return Ok(DeleteJobOutcome::NotFound);
        }
        if job.status.is_active() {
            return Ok(DeleteJobOutcome::Active);
        }
        jobs.remove(id.as_str())?;
        txn.open_table(RESULTS)?.remove(id.as_str())?;
    }
    txn.commit()?;
    Ok(DeleteJobOutcome::Deleted)
}

pub fn prune(db: &Database) -> Result<()> {
    let txn = db.begin_write()?;
    {
        let mut jobs = txn.open_table(JOBS)?;
        let mut results = txn.open_table(RESULTS)?;
        let now = Utc::now().timestamp();
        let expired = jobs
            .iter()?
            .map(|row| -> Result<Job> { Ok(serde_json::from_str(row?.1.value())?) })
            .collect::<Result<Vec<_>>>()?;
        for job in expired.into_iter().filter(|job| job.expired(now)) {
            jobs.remove(job.id.to_string().as_str())?;
            results.remove(job.id.to_string().as_str())?;
        }
    }
    txn.commit()?;
    Ok(())
}

pub fn key_by_digest(db: &Database, digest: &str) -> Result<Option<Key>> {
    let txn = db.begin_read()?;
    let keys = txn.open_table(KEYS)?;
    keys.get(digest)?
        .map(|value| serde_json::from_str(value.value()).map_err(Into::into))
        .transpose()
}

pub fn list_keys(db: &Database) -> Result<Vec<Key>> {
    let txn = db.begin_read()?;
    let keys = txn.open_table(KEYS)?;
    let mut list = keys
        .iter()?
        .map(|row| Ok(serde_json::from_str::<Key>(row?.1.value())?))
        .collect::<Result<Vec<_>>>()?;
    list.sort_by_key(|key| std::cmp::Reverse((key.created_at, key.id)));
    Ok(list)
}

pub fn insert_key(db: &Database, digest: &str, key: &Key) -> Result<bool> {
    let txn = db.begin_write()?;
    {
        let mut keys = txn.open_table(KEYS)?;
        if keys.len()? >= 100 {
            return Ok(false);
        }
        keys.insert(digest, serde_json::to_string(key)?.as_str())?;
    }
    txn.commit()?;
    Ok(true)
}

pub fn revoke_key(db: &Database, id: Uuid) -> Result<bool> {
    let txn = db.begin_write()?;
    {
        let mut keys = txn.open_table(KEYS)?;
        let mut digest = None;
        for row in keys.iter()? {
            let (stored_digest, value) = row?;
            if serde_json::from_str::<Key>(value.value())?.id == id {
                digest = Some(stored_digest.value().to_owned());
                break;
            }
        }
        let Some(digest) = digest else {
            return Ok(false);
        };
        keys.remove(digest.as_str())?;
    }
    txn.commit()?;
    Ok(true)
}
