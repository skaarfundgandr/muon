#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use muon::infrastructure::storage::pool::{create_pool, init_pool};

table! {
    pragma_journal_mode (journal_mode) {
        journal_mode -> Text,
    }
}

table! {
    count_result (c) {
        c -> BigInt,
    }
}

#[derive(QueryableByName)]
#[diesel(table_name = pragma_journal_mode)]
struct JournalMode {
    journal_mode: String,
}

#[derive(QueryableByName)]
#[diesel(table_name = count_result)]
struct CountRow {
    c: i64,
}

fn temp_db_path() -> String {
    let mut p = std::env::temp_dir();
    p.push(format!("muon-pool-test-{}.db", uuid::Uuid::new_v4()));
    let s = p.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&s);
    s
}

fn cleanup(path: &str) {
    let _ = std::fs::remove_file(path);
    let p = std::path::Path::new(path);
    let _ = std::fs::remove_file(p.with_extension("db-wal"));
    let _ = std::fs::remove_file(p.with_extension("db-shm"));
}

#[tokio::test]
async fn create_pool_applies_wal_journal_mode() {
    let path = temp_db_path();
    let pool = create_pool(&path).unwrap();
    let mut conn = pool.get().await.unwrap();
    let row: JournalMode = diesel::sql_query("PRAGMA journal_mode")
        .get_result(&mut *conn)
        .await
        .unwrap();
    assert_eq!(row.journal_mode.to_lowercase(), "wal");
    drop(conn);
    drop(pool);
    cleanup(&path);
}

#[tokio::test]
async fn create_pool_concurrent_writers_no_deadlock() {
    let path = temp_db_path();
    let pool = create_pool(&path).unwrap();

    diesel::sql_query("CREATE TABLE t (id INTEGER PRIMARY KEY)")
        .execute(&mut *pool.get().await.unwrap())
        .await
        .unwrap();

    let mut handles = Vec::new();
    for i in 0..8 {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            let mut conn = pool.get().await.unwrap();
            diesel::sql_query(format!("INSERT INTO t (id) VALUES ({i})"))
                .execute(&mut *conn)
                .await
                .unwrap();
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let mut conn = pool.get().await.unwrap();
    let count: i64 = diesel::sql_query("SELECT COUNT(*) AS c FROM t")
        .get_result::<CountRow>(&mut *conn)
        .await
        .unwrap()
        .c;
    assert_eq!(count, 8);
    drop(conn);
    drop(pool);
    cleanup(&path);
}

#[tokio::test]
async fn init_pool_is_idempotent_for_same_path() {
    let path = temp_db_path();
    let p1 = init_pool(&path).await.unwrap();
    let p2 = init_pool(&path).await.unwrap();
    assert!(p1.get().await.is_ok());
    assert!(p2.get().await.is_ok());
    drop(p1);
    drop(p2);
    cleanup(&path);
}
