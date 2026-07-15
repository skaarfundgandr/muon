#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::infrastructure::observability::otel_attr_value_with;

#[test]
fn otel_attr_full_when_debug() {
    let big = "x".repeat(50_000);
    assert_eq!(otel_attr_value_with(&big, true).len(), 50_000);
}

#[test]
fn otel_attr_caps_when_not_debug_and_huge() {
    let big = "y".repeat(50_000);
    let out = otel_attr_value_with(&big, false);
    assert!(out.chars().count() < 50_000);
    assert!(out.ends_with('…'));
}

#[test]
fn otel_attr_small_unchanged_when_not_debug() {
    assert_eq!(otel_attr_value_with("hello", false), "hello");
}
