use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;

use crate::error::MuonError;
use crate::infrastructure::storage::migrations::run_migrations;

pub type DbPool = Pool<SyncConnectionWrapper<SqliteConnection>>;

pub async fn init_pool(path: &str) -> Result<DbPool, MuonError> {
    let expanded = crate::infrastructure::util::expand_tilde(path);
    if let Some(parent) = expanded.parent() {
        std::fs::create_dir_all(parent).map_err(|e| MuonError::Database(e.to_string()))?;
    }
    let path_str = expanded.to_string_lossy().to_string();
    let mut sync_conn = SqliteConnection::establish(&path_str)
        .map_err(|e| MuonError::Database(e.to_string()))?;
    run_migrations(&mut sync_conn)?;

    let mgr = AsyncDieselConnectionManager::<SyncConnectionWrapper<SqliteConnection>>::new(path_str);
    let pool = Pool::builder(mgr)
        .max_size(8)
        .build()
        .map_err(|e| MuonError::Database(e.to_string()))?;
    Ok(pool)
}

