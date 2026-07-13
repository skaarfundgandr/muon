#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::presentation::components::inputs::query_input::{QueryInput, visible_around_caret};

#[test]
fn short_query_shows_all() {
    let (pre, post, scrolled) = visible_around_caret("hello", 5, 40);
    assert_eq!(pre, "hello");
    assert_eq!(post, "");
    assert!(!scrolled);
}

#[test]
fn long_query_follows_caret_at_end() {
    let buf: String = (0..80).map(|_| 'a').collect();
    let (pre, post, scrolled) = visible_around_caret(&buf, buf.len(), 20);
    assert!(scrolled);
    assert_eq!(post, "");
    assert_eq!(pre.chars().count() + post.chars().count(), 20);
    assert!(pre.ends_with('a'));
    assert!(pre.chars().count() <= 20);
    assert!(pre.chars().count() >= 20 - 3);
}

#[test]
fn long_query_follows_caret_at_start() {
    let buf: String = (0..80).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
    let (pre, post, scrolled) = visible_around_caret(&buf, 0, 20);
    assert!(!scrolled);
    assert_eq!(pre, "");
    assert_eq!(post.chars().count(), 20);
}

#[test]
fn mid_caret_window_contains_caret() {
    let buf: String = (0..100).map(|_| 'x').collect();
    let cursor = 50;
    let (pre, post, scrolled) = visible_around_caret(&buf, cursor, 20);
    assert!(scrolled);
    assert_eq!(pre.chars().count() + post.chars().count(), 20);
    assert!(!pre.is_empty() || !post.is_empty());
}

#[test]
fn insert_and_cursor_utf8_boundaries() {
    let mut q = QueryInput {
        active: true,
        ..Default::default()
    };
    q.insert_char('a');
    q.insert_char('b');
    q.insert_char('c');
    assert_eq!(q.buffer, "abc");
    assert_eq!(q.cursor, 3);
    q.cursor_left();
    assert_eq!(q.cursor, 2);
    q.backspace();
    assert_eq!(q.buffer, "ac");
    assert_eq!(q.cursor, 1);
    q.insert_char('日');
    assert_eq!(q.buffer, "a日c");
    assert_eq!(q.cursor, 1 + '日'.len_utf8());
    q.cursor_left();
    assert_eq!(q.cursor, 1);
    q.delete();
    assert_eq!(q.buffer, "ac");
}
