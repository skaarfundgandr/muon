#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use muon::application::config::MuonConfig;
use muon::domain::error::MuonError;
use muon::domain::models::source::{Source, SourceType, VerificationStatus};
use muon::domain::traits::vector_store::VectorStore;
use muon::infrastructure::rag::rag_store::RagContext;

#[tokio::test]
#[ignore]
async fn rag_round_trip() -> Result<(), MuonError> {
    let tmp = tempfile::tempdir().map_err(|e| MuonError::Database(e.to_string()))?;
    let db = tmp.path().join("rag.db");

    let mut cfg = MuonConfig::default();
    cfg.advanced.rag_db_path = db.to_string_lossy().into_owned();

    let ctx = RagContext::open(&cfg).await?;

    let source = Source {
        url: "test://example.com".to_string(),
        title: "Test Doc".to_string(),
        snippet: String::new(),
        relevance: 1.0,
        source_type: SourceType::Knowledge,
        verified: false,
        verification_status: VerificationStatus::Unverified,
        embedding_id: None,
    };

    let id = ctx
        .add(&source, "Hello world, this is a test document.")
        .await?;
    assert!(id.is_some());

    let results = ctx.query("test document", 5).await?;
    assert!(!results.is_empty());
    assert_eq!(results[0].url, "test://example.com");
    Ok(())
}

#[test]
fn temp_rag_path_caps_long_url_filename() {
    use muon::infrastructure::rag::temp_rag_path;

    let long = format!("https://example.com/{}", "a".repeat(500));
    let path = temp_rag_path(&long);
    let name = path.file_name().and_then(|n| n.to_str()).unwrap();
    assert!(
        name.len() <= 200,
        "filename too long for common NAME_MAX: {} ({name})",
        name.len()
    );
    assert!(name.starts_with("muon-rag-"));
    assert!(name.ends_with(".txt"));
    // write must succeed even when URL is huge
    std::fs::write(&path, b"x").unwrap();
    let _ = std::fs::remove_file(&path);
}

#[test]
fn temp_rag_path_empty_url_still_valid() {
    use muon::infrastructure::rag::temp_rag_path;

    let path = temp_rag_path("!!!");
    let name = path.file_name().and_then(|n| n.to_str()).unwrap();
    assert!(name.starts_with("muon-rag-src-") || name.contains("muon-rag-"));
}
