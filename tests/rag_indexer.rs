#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use muon::application::services::RagIndexerService;
use muon::domain::error::MuonError;
use muon::domain::models::source::Source;
use muon::domain::traits::vector_store::VectorStore;

/// Stub VectorStore that records add calls without embedding.
#[derive(Debug, Default)]
struct CountingStore {
    calls: std::sync::Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl VectorStore for CountingStore {
    async fn add(&self, source: &Source, content: &str) -> Result<usize, MuonError> {
        let mut calls = self.calls.lock().unwrap();
        calls.push((source.url.clone(), content.to_string()));
        Ok(1)
    }

    async fn query(&self, _text: &str, _k: usize) -> Result<Vec<Source>, MuonError> {
        Ok(Vec::new())
    }

    async fn save_index(&self) -> Result<(), MuonError> {
        Ok(())
    }
}

fn fixture_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("muon-rag-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, content).unwrap();
    p
}

fn create_pdf_bytes(texts: &[&str]) -> Vec<u8> {
    let mut md = String::new();
    for (i, t) in texts.iter().enumerate() {
        if i > 0 {
            md.push_str("\n\n\\newpage\n\n");
        }
        md.push_str(t);
    }
    use pdf_oxide::api::Pdf;
    let mut pdf = Pdf::from_markdown(&md).unwrap();
    pdf.to_bytes().unwrap()
}

#[test]
fn indexer_indexes_txt_file() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = fixture_dir("txt_file");
    write_file(&dir, "hello.txt", "Hello, world!");

    let store = Arc::new(CountingStore::default());

    rt.block_on(async {
        let summary = RagIndexerService::index(
            store.as_ref() as &dyn VectorStore,
            &dir.join("hello.txt"),
            "FILE",
        )
        .await;

        assert_eq!(summary.total_files, 1, "should index 1 file");
        assert_eq!(summary.total_chunks, 1, "should produce 1 chunk");
        assert!(summary.errors.is_empty(), "errors: {:?}", summary.errors);
    });

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn indexer_indexes_directory_recursively() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = fixture_dir("dir_recurse");
    write_file(&dir, "a.txt", "File A");
    let sub = dir.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    write_file(&sub, "b.md", "File B");

    let store = Arc::new(CountingStore::default());

    rt.block_on(async {
        let summary =
            RagIndexerService::index(store.as_ref() as &dyn VectorStore, &dir, "DIRECTORY").await;

        assert_eq!(summary.total_files, 2, "should index 2 files");
        assert_eq!(summary.total_chunks, 2, "should produce 2 chunks");
        assert!(summary.errors.is_empty(), "errors: {:?}", summary.errors);
    });

    let calls = store.calls.lock().unwrap();
    let urls: Vec<&str> = calls.iter().map(|(u, _)| u.as_str()).collect();
    assert!(urls.iter().any(|u| u.contains("a.txt")), "should contain a.txt");
    assert!(urls.iter().any(|u| u.contains("b.md")), "should contain b.md");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn indexer_indexes_pdf_file() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = fixture_dir("pdf_file");

    let pdf_bytes = create_pdf_bytes(&["# Page One", "Content on page two"]);
    let pdf_path = dir.join("test.pdf");
    std::fs::write(&pdf_path, &pdf_bytes).unwrap();

    let store = Arc::new(CountingStore::default());

    rt.block_on(async {
        let summary =
            RagIndexerService::index(store.as_ref() as &dyn VectorStore, &pdf_path, "FILE").await;

        assert_eq!(summary.total_files, 1, "should index 1 PDF");
        assert_eq!(summary.total_chunks, 1, "should produce 1 chunk");
        assert!(summary.errors.is_empty(), "errors: {:?}", summary.errors);

        // Verify multi-page content was captured
        let calls = store.calls.lock().unwrap();
        let content = &calls[0].1;
        assert!(content.contains("Page One"), "should contain first page content");
        assert!(content.contains("page two"), "should contain second page content");
    });

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn indexer_skips_unsupported_extensions() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = fixture_dir("skip_ext");
    write_file(&dir, "data.csv", "a,b,c\n1,2,3");
    write_file(&dir, "readme.txt", "Hello");

    let store = Arc::new(CountingStore::default());

    rt.block_on(async {
        let summary =
            RagIndexerService::index(store.as_ref() as &dyn VectorStore, &dir, "DIRECTORY").await;

        assert_eq!(summary.total_files, 1, "only .txt should be indexed");
        assert_eq!(summary.total_chunks, 1);
        assert!(summary.errors.is_empty(), "errors: {:?}", summary.errors);
    });

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn indexer_glob_pattern_indexes_only_matching_files() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = fixture_dir("glob_test");
    write_file(&dir, "a.md", "# Markdown A");
    write_file(&dir, "b.txt", "Text B");
    write_file(&dir, "c.py", "print('hello')");

    let pattern = dir.join("*.md").to_string_lossy().to_string();

    let store = Arc::new(CountingStore::default());

    rt.block_on(async {
        let summary = RagIndexerService::index(
            store.as_ref() as &dyn VectorStore,
            std::path::Path::new(&pattern),
            "GLOB",
        )
        .await;

        assert_eq!(summary.total_files, 1, "only .md should be indexed by glob");
        assert_eq!(summary.total_chunks, 1, "should produce 1 chunk");
        assert!(summary.errors.is_empty(), "errors: {:?}", summary.errors);
    });

    let calls = store.calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "only one file should have been added");
    assert!(calls[0].0.contains("a.md"), "should be a.md");

    let _ = std::fs::remove_dir_all(&dir);
}
