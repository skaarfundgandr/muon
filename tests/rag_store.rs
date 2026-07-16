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

    let n = ctx
        .add(&source, "Hello world, this is a test document.")
        .await?;
    assert!(n > 0);

    let results = ctx.query("test document", 5).await?;
    assert!(!results.is_empty());
    assert_eq!(results[0].url, "test://example.com");
    Ok(())
}

#[test]
fn temp_rag_path_is_short() {
    use muon::infrastructure::rag::temp_rag_path;
    let path = temp_rag_path(&format!("https://example.com/{}", "a".repeat(500)));
    let name = path.file_name().and_then(|n| n.to_str()).unwrap();
    assert!(name.len() < 80, "got {name}");
    assert!(name.starts_with("muon-rag-"));
}

#[test]
fn pack_unpack_rag_content_round_trip() {
    use muon::infrastructure::rag::{pack_rag_content, unpack_rag_content};
    let packed = pack_rag_content(
        "https://example.com/paper",
        "My Paper Title",
        "snippet body here",
    );
    let (url, title, body) = unpack_rag_content(&packed);
    assert_eq!(url.as_deref(), Some("https://example.com/paper"));
    assert_eq!(title.as_deref(), Some("My Paper Title"));
    assert_eq!(body, "snippet body here");
}

#[test]
fn unpack_plain_content_unchanged() {
    use muon::infrastructure::rag::unpack_rag_content;
    let (url, title, body) = unpack_rag_content("just a snippet");
    assert!(url.is_none());
    assert!(title.is_none());
    assert_eq!(body, "just a snippet");
}

#[test]
fn pack_rag_escapes_gt_in_url_title() {
    use muon::infrastructure::rag::{pack_rag_content, unpack_rag_content};
    let packed = pack_rag_content(
        "https://example.com/a>>>b",
        "title with >>> inside",
        "body with >>> preserved",
    );
    let (url, title, body) = unpack_rag_content(&packed);
    assert_eq!(url.as_deref(), Some("https://example.com/a   b"));
    assert_eq!(title.as_deref(), Some("title with     inside"));
    assert_eq!(body, "body with >>> preserved");
}
