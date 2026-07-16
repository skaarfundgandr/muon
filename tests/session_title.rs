#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::session::derive_title;

#[test]
fn short_ascii_capitalizes_first_char_no_ellipsis() {
    assert_eq!(derive_title("hello world"), "Hello world");
}

#[test]
fn long_ascii_ends_with_ellipsis() {
    let long = "aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee";
    let result = derive_title(long);
    assert!(result.ends_with("..."), "result: {result:?}");
    assert_eq!(result.chars().count(), 40, "37 chars + ... = 40");
}

#[test]
fn long_cjk_emoji_no_panic() {
    let long = "🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀 🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀 🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀 🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀 🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀";
    let result = derive_title(long);
    assert!(result.ends_with("..."), "result: {result:?}");
    assert_eq!(result.chars().count(), 40, "37 chars + ... = 40");
}

#[test]
fn empty_returns_untitled_session() {
    assert_eq!(derive_title(""), "Untitled Session");
}

#[test]
fn whitespace_only_returns_untitled_session() {
    assert_eq!(derive_title("   "), "Untitled Session");
}
