#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::domain::error::MuonError;
use muon::infrastructure::agent_rs::{
    classify_and_render, classify_body, html_bytes_to_output, pdf_bytes_to_text, BodyKind,
};

const MAX_BODY_BYTES: usize = 2_000_000;

#[test]
fn test_html_script_style_title_not_in_output() {
    let html = b"<!DOCTYPE html><html><head>
        <title>Foo</title>
        <style>body { color: red; }</style>
        <link rel='stylesheet' href='style.css'>
    </head><body>
        <script>function() { return 1; }</script>
        <p>Hello World</p>
    </body></html>";
    let (text, title) = html_bytes_to_output(html, 10_000);
    assert_eq!(title.as_deref(), Some("Foo"));
    assert!(text.contains("Hello World"), "expected markdown text to contain paragraph content");
    assert!(!text.contains("function()"), "script content should not be in output");
    assert!(!text.contains("color: red"), "style content should not be in output");
}

#[test]
fn test_html_truncation_honors_max_chars() {
    let long_text = "a".repeat(10_000)
        + &"b".repeat(10_000)
        + &"c".repeat(10_000);
    let html = format!("<html><body><p>{}</p></body></html>", long_text);
    let (text, _title) = html_bytes_to_output(html.as_bytes(), 5_000);
    assert!(text.chars().count() <= 5_000, "output should be truncated to at most 5000 chars");
}

#[test]
fn test_synthetic_pdf_classify_and_extract() {
    let mut pdf = pdf_oxide::api::Pdf::from_markdown("# Hello\n\nThis is a World phrase MARKER.\n")
        .unwrap();
    let bytes = pdf.save_to_bytes().unwrap();

    assert_eq!(classify_body(None, &bytes), BodyKind::Pdf);
    assert_eq!(classify_body(Some("application/pdf"), &bytes), BodyKind::Pdf);

    let text = pdf_bytes_to_text(&bytes, 100_000).unwrap();
    assert!(text.contains("Hello"), "extracted text should contain 'Hello'");
    assert!(text.contains("World"), "extracted text should contain 'World'");
    assert!(text.contains("MARKER"), "extracted text should contain MARKER phrase");
    assert!(!text.contains('\u{FFFD}'), "extracted text should NOT contain replacement characters");
}

#[test]
fn test_pdf_magic_overrides_wrong_content_type() {
    let bytes = b"%PDF-1.4\n1 0 obj\n<</Type/Catalog>>\nendobj\n%%EOF";
    assert_eq!(
        classify_body(Some("text/html"), bytes),
        BodyKind::Pdf,
        "%PDF- magic must override text/html content type"
    );
}

#[test]
fn test_random_octet_stream_is_unsupported() {
    let bytes = [0x00, 0x01, 0x02, 0x03];
    assert_eq!(
        classify_body(Some("application/octet-stream"), &bytes),
        BodyKind::Unsupported
    );

    let result = classify_and_render(Some("application/octet-stream"), &bytes, false, 10_000);
    assert!(
        matches!(result, Err(MuonError::Search { .. })),
        "unsupported content type should return Search error"
    );
}

#[test]
fn test_pdf_truncated_returns_clear_error() {
    let mut bytes = b"%PDF-".to_vec();
    bytes.resize(MAX_BODY_BYTES + 1, 0xFF);

    assert_eq!(classify_body(Some("application/pdf"), &bytes), BodyKind::Pdf);

    let result = classify_and_render(Some("application/pdf"), &bytes, true, 10_000);
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("PDF too large"),
        "error message should mention 'PDF too large', got: {msg}"
    );
}

#[test]
fn test_truncated_non_pdf_still_classified_as_html() {
    let mut bytes = b"<html>".to_vec();
    bytes.resize(MAX_BODY_BYTES, 0x00);

    assert_eq!(
        classify_body(Some("text/html"), &bytes),
        BodyKind::Html,
        "HTML body over cap should still classify as Html (only PDF fails closed on truncation)"
    );
}
