#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use muon::application::services::slugify;

#[test]
fn test_slugify() {
    assert_eq!(slugify("Hello World"), "hello-world");
    assert_eq!(slugify("My Cool Report!!!"), "my-cool-report");
    assert_eq!(slugify("a".repeat(100).as_str()), "a".repeat(60));
}
