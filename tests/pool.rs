#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use muon::infrastructure::storage::pool::init_pool;

#[tokio::test]
async fn init_pool_memory_returns_ok() {
    let result = init_pool(":memory:").await;
    assert!(result.is_ok());
}
