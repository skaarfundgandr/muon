#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::net::IpAddr;

use muon::infrastructure::agent_rs::{
    ensure_public_resolved, is_blocked_ip, is_public_http_url,
};

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

// --- is_blocked_ip ---

#[test]
fn blocked_ip_loopback_v4() {
    let ip: IpAddr = "127.0.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_loopback_v6() {
    let ip: IpAddr = "::1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_private_10() {
    let ip: IpAddr = "10.0.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_private_192_168() {
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_private_172_16() {
    let ip: IpAddr = "172.16.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_link_local() {
    let ip: IpAddr = "169.254.1.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_unspecified_v4() {
    let ip: IpAddr = "0.0.0.0".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_unspecified_v6() {
    let ip: IpAddr = "::".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_cgnat() {
    let ip: IpAddr = "100.64.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
    let ip: IpAddr = "100.127.255.255".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_multicast() {
    let ip: IpAddr = "224.0.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_broadcast() {
    let ip: IpAddr = "255.255.255.255".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn allowed_public_ip() {
    let ip: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(!is_blocked_ip(ip));
}

#[test]
fn blocked_ip_v4_mapped_loopback() {
    let ip: IpAddr = "::ffff:127.0.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocked_ip_v4_mapped_private() {
    let ip: IpAddr = "::ffff:10.0.0.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
    let ip: IpAddr = "::ffff:192.168.1.1".parse().unwrap();
    assert!(is_blocked_ip(ip));
}

#[test]
fn blocks_v4_mapped_url_literal() {
    assert!(is_public_http_url("http://[::ffff:127.0.0.1]/").is_err());
    assert!(is_public_http_url("http://[::ffff:10.0.0.1]/").is_err());
}

// --- ensure_public_resolved ---

#[tokio::test]
async fn resolve_blocks_loopback_v4() {
    assert!(ensure_public_resolved("127.0.0.1").await.is_err());
}

#[tokio::test]
async fn resolve_blocks_private_10() {
    assert!(ensure_public_resolved("10.0.0.1").await.is_err());
}

#[tokio::test]
async fn resolve_blocks_private_192_168() {
    assert!(ensure_public_resolved("192.168.1.1").await.is_err());
}

#[tokio::test]
async fn resolve_blocks_private_172_16() {
    assert!(ensure_public_resolved("172.16.0.1").await.is_err());
}

#[tokio::test]
async fn resolve_blocks_loopback_v6() {
    assert!(ensure_public_resolved("::1").await.is_err());
}

#[tokio::test]
async fn resolve_blocks_unspecified() {
    assert!(ensure_public_resolved("0.0.0.0").await.is_err());
}
