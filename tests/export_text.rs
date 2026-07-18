#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::services::soft_wrap_markdown_for_pdf;

#[test]
fn wrap_drops_height_96_to_72_budget() {
    // Long prose line longer than the new 72-char budget wraps onto multiple
    // lines; no individual output line exceeds the budget.
    let line = "a".repeat(200);
    let wrapped = soft_wrap_markdown_for_pdf(&line, 72);
    for out_line in wrapped.lines() {
        assert!(
            out_line.chars().count() <= 72,
            "output line should not exceed 72 chars, got: {out_line}"
        );
    }
    assert!(
        wrapped.lines().count() >= 3,
        "200-char input should wrap into at least 3 lines at width 72"
    );
}

#[test]
fn wrap_replaces_dash_dividers_with_em_dash_row() {
    let md = "para one\n\n---\n\npara two";
    let wrapped = soft_wrap_markdown_for_pdf(md, 72);
    assert!(
        !wrapped.contains("\n---\n") && !wrapped.contains("\n----\n"),
        "standalone dash divider must not appear verbatim, got: {wrapped}"
    );
    let em_row_count = wrapped
        .lines()
        .filter(|l| l.chars().all(|c| c == '\u{2014}') && !l.is_empty())
        .count();
    assert_eq!(
        em_row_count, 1,
        "exactly one em-dash divider row expected, got: {wrapped}"
    );
}

#[test]
fn wrap_preserves_short_dash_runs_inside_prose() {
    // A line that merely contains dashes mid-text (not 3+ dash-only) is
    // not a divider and must pass through as ordinary text.
    let md = "the value was high — very high — and falling";
    let wrapped = soft_wrap_markdown_for_pdf(md, 72);
    assert!(
        wrapped.contains("the value was high"),
        "prose with em-dashes must be preserved, got: {wrapped}"
    );
}

#[test]
fn wrap_wraps_long_atx_heading_text() {
    // An H1 whose text body exceeds the budget should wrap so it no longer
    // clips the right margin. Continuation lines must be indented under the
    // heading text column (no `# ` prefix on continuation lines, since
    // pdf_oxide would otherwise treat each one as an H1 of its own).
    let h1 = format!("# {}", "Using Satellite Data for Monitoring Air Pollution TROPOMI Caps");
    let wrapped = soft_wrap_markdown_for_pdf(&h1, 40);
    let lines: Vec<&str> = wrapped.lines().collect();
    assert!(lines.len() >= 2, "heading should wrap at width 40, got: {wrapped}");
    assert!(
        lines[0].starts_with("# "),
        "first wrapped line must keep the ATX prefix"
    );
    for cont in &lines[1..] {
        assert!(
            !cont.starts_with('#'),
            "continuation lines must not re-emit '#': {cont:?}"
        );
    }
}

#[test]
fn wrap_preserves_short_headings_unchanged() {
    let md = "## Topic 1\n\nbody text";
    let wrapped = soft_wrap_markdown_for_pdf(md, 72);
    assert!(
        wrapped.contains("## Topic 1"),
        "short heading must be preserved verbatim, got: {wrapped}"
    );
}

#[test]
fn wrap_does_not_touch_fenced_code_blocks() {
    let md = "```\n---\nvery-long-line-without-breaks\n```";
    let wrapped = soft_wrap_markdown_for_pdf(md, 72);
    assert!(
        wrapped.contains("---\n"),
        "fenced code block line `---` must NOT be replaced with em-dash row, got: {wrapped}"
    );
    assert!(
        wrapped.contains("very-long-line-without-breaks"),
        "fenced code block content must be left intact, got: {wrapped}"
    );
}

#[test]
fn wrap_hard_breaks_overlong_heading_token() {
    let token = "https://example.com/".to_string() + &"a".repeat(80);
    let h1 = format!("# {token}");
    let wrapped = soft_wrap_markdown_for_pdf(&h1, 40);
    let lines: Vec<&str> = wrapped.lines().collect();
    assert!(lines.len() >= 2, "overlong heading token must hard-break, got: {wrapped}");
    assert!(lines[0].starts_with("# "), "first line keeps ATX prefix");
    for line in &lines {
        assert!(
            line.chars().count() <= 40,
            "no heading line may exceed width 40: {line:?}"
        );
    }
    for cont in &lines[1..] {
        assert!(!cont.starts_with('#'), "continuation must not re-emit '#': {cont:?}");
        assert!(
            cont.starts_with("  "),
            "continuation must be indented under heading text: {cont:?}"
        );
    }
}

#[test]
fn wrap_preserves_list_indent_on_continuation() {
    let body = "word ".repeat(30);
    let md = format!("- {body}");
    let wrapped = soft_wrap_markdown_for_pdf(&md, 40);
    let lines: Vec<&str> = wrapped.lines().collect();
    assert!(lines.len() >= 2, "long list item must wrap, got: {wrapped}");
    assert!(
        lines[0].starts_with("- "),
        "first line keeps list marker, got: {:?}",
        lines[0]
    );
    for cont in &lines[1..] {
        assert!(
            cont.starts_with("  "),
            "list continuation must keep indent under marker, got: {cont:?}"
        );
        assert!(
            !cont.trim_start().starts_with("- "),
            "continuation must not re-emit list marker: {cont:?}"
        );
    }
}
