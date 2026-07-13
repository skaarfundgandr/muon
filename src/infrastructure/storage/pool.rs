use std::sync::{Mutex, OnceLock};

use diesel::Connection;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use diesel_async::SimpleAsyncConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::{Hook, HookError, Pool};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;

use crate::domain::error::MuonError;
use crate::infrastructure::storage::migrations::run_migrations;

pub type DbPool = Pool<SyncConnectionWrapper<SqliteConnection>>;

static INIT_LOCK: Mutex<()> = Mutex::new(());
static SESSION_POOL: OnceLock<DbPool> = OnceLock::new();
static SESSION_POOL_PATH: OnceLock<String> = OnceLock::new();

const SQLITE_PRAGMAS: &str = "\
PRAGMA foreign_keys = ON;\
PRAGMA journal_mode = WAL;\
PRAGMA synchronous = NORMAL;\
PRAGMA busy_timeout = 5000;\
PRAGMA mmap_size = 30000000000;";

fn expand_and_ensure_parent(path: &str) -> Result<String, MuonError> {
    let expanded = crate::infrastructure::util::expand_tilde(path);
    if let Some(parent) = expanded.parent() {
        std::fs::create_dir_all(parent).map_err(|e| MuonError::Database(e.to_string()))?;
    }
    Ok(expanded.to_string_lossy().to_string())
}

pub fn create_pool(path_str: &str) -> Result<DbPool, MuonError> {
    let mgr = AsyncDieselConnectionManager::<SyncConnectionWrapper<SqliteConnection>>::new(
        path_str.to_string(),
    );
    let builder = Pool::builder(mgr).max_size(8).post_create(Hook::async_fn(
        |conn: &mut SyncConnectionWrapper<SqliteConnection>, _| {
            Box::pin(async move {
                conn.batch_execute(SQLITE_PRAGMAS)
                    .await
                    .map_err(|e| HookError::Message(e.to_string().into()))?;
                Ok(())
            })
        },
    ));
    builder
        .build()
        .map_err(|e| MuonError::Database(e.to_string()))
}

pub async fn init_pool(path: &str) -> Result<DbPool, MuonError> {
    let path_str = expand_and_ensure_parent(path)?;

    // Fast path: lock-free check — no mutex contention on the hot path.
    if let Some(existing) = SESSION_POOL.get() {
        if let Some(prev) = SESSION_POOL_PATH.get()
            && prev != &path_str
        {
            return Err(MuonError::Config(format!(
                "session DB path changed from {prev} to {path_str}; restart muon to apply"
            )));
        }
        return Ok(existing.clone());
    }

    // Slow path: serialize against intra-process TOCTOU races.
    let _guard = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Double-checked locking: another thread may have finished init while
    // we waited on the mutex.
    if let Some(existing) = SESSION_POOL.get() {
        if let Some(prev) = SESSION_POOL_PATH.get()
            && prev != &path_str
        {
            return Err(MuonError::Config(format!(
                "session DB path changed from {prev} to {path_str}; restart muon to apply"
            )));
        }
        return Ok(existing.clone());
    }

    let mut sync =
        SqliteConnection::establish(&path_str).map_err(|e| MuonError::Database(e.to_string()))?;
    sync.batch_execute(SQLITE_PRAGMAS)
        .map_err(|e| MuonError::Database(e.to_string()))?;
    run_migrations(&mut sync)?;
    drop(sync);

    let pool = create_pool(&path_str)?;
    let _ = SESSION_POOL_PATH.set(path_str);
    let _ = SESSION_POOL.set(pool);
    SESSION_POOL
        .get()
        .cloned()
        .ok_or_else(|| MuonError::Database("session pool missing after init".into()))
}

pub fn global_pool() -> Result<DbPool, MuonError> {
    SESSION_POOL
        .get()
        .cloned()
        .ok_or_else(|| MuonError::Database("session pool not initialized".into()))
}

pub async fn open_pool(path: &str) -> Result<DbPool, MuonError> {
    let path_str = expand_and_ensure_parent(path)?;
    let mut sync =
        SqliteConnection::establish(&path_str).map_err(|e| MuonError::Database(e.to_string()))?;
    sync.batch_execute(SQLITE_PRAGMAS)
        .map_err(|e| MuonError::Database(e.to_string()))?;
    run_migrations(&mut sync)?;
    drop(sync);
    create_pool(&path_str)
}
