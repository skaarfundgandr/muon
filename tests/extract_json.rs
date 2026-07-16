#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::domain::extract_json;

#[test]
fn test_extract_json_code_fence() {
    assert_eq!(
        extract_json("```json\n{\"foo\": \"bar\"}\n```"),
        Some("{\"foo\": \"bar\"}")
    );
    assert_eq!(
        extract_json("```\n{\"foo\": \"bar\"}\n```"),
        Some("{\"foo\": \"bar\"}")
    );
}

#[test]
fn test_extract_json_no_fence() {
    assert_eq!(
        extract_json("{\"foo\": \"bar\"}"),
        Some("{\"foo\": \"bar\"}")
    );
}

#[test]
fn test_extract_json_surrounding_prose() {
    assert_eq!(
        extract_json("Here is some json: {\"foo\": \"bar\"} and some trailing text."),
        Some("{\"foo\": \"bar\"}")
    );
}

#[test]
fn test_extract_json_case_insensitive_fence() {
    assert_eq!(
        extract_json("```JSON\n{\"foo\": \"bar\"}\n```"),
        Some("{\"foo\": \"bar\"}")
    );
    assert_eq!(
        extract_json("```Json\n{\"foo\": \"bar\"}\n```"),
        Some("{\"foo\": \"bar\"}")
    );
    assert_eq!(
        extract_json("```json\n{\"foo\": \"bar\"}\n```"),
        Some("{\"foo\": \"bar\"}")
    );
    assert_eq!(
        extract_json("```\n{\"foo\": \"bar\"}\n```"),
        Some("{\"foo\": \"bar\"}")
    );
}

#[test]
fn test_extract_json_invalid() {
    assert_eq!(extract_json("no json here"), None);
    assert_eq!(extract_json(""), None);
}
