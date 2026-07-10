#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::infrastructure::agent_rs::is_public_http_url;

#[test]
fn allows_https_public() {
    assert!(is_public_http_url("https://example.com/page").is_ok());
}

#[test]
fn blocks_localhost() {
    assert!(is_public_http_url("http://127.0.0.1/").is_err());
    assert!(is_public_http_url("http://localhost/").is_err());
}

#[test]
fn blocks_metadata() {
    assert!(is_public_http_url("http://169.254.169.254/latest").is_err());
}

#[test]
fn blocks_file_scheme() {
    assert!(is_public_http_url("file:///etc/passwd").is_err());
}
