/// Soft-wrap markdown source at `width` (character budget) for PDF rendering.
///
/// pdf_oxide's `from_markdown` lays out one source line → one PDF line and
/// never wraps on its own, so any source line longer than the printable text
/// width overflows the right margin and clips. This helper pre-wraps source
/// lines to a safe character budget and expands standalone thematic-break
/// lines (`---`, `----`, …) into a visible horizontal divider.
///
/// - Fenced code blocks (lines between ``` fences) are passed through
///   unwrapped (D18).
/// - Blank lines and ATX heading lines (`# `, `## `, …) are wrapped in a
///   heading-aware way: the prefix is preserved on the first wrapped line
///   and continuation lines are indented under the heading's text column so
///   they visually continue the heading even though pdf_oxide renders each
///   source line as its own PDF line.
/// - Standalone divider lines (3+ dashes, nothing else) are replaced with a
///   row of em-dashes sized to the width budget, which pdf_oxide renders as
///   a visible horizontal rule.
/// - Overlong single tokens (URLs, paths) break on `/ ? & =` then hard-chunk
///   at the width.
pub fn soft_wrap_markdown_for_pdf(md: &str, width: usize) -> String {
    if width == 0 {
        return md.to_string();
    }

    let mut out = String::new();
    let mut in_fence = false;

    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if in_fence {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if line.trim().is_empty() {
            out.push('\n');
            continue;
        }

        if is_divider_line(line) {
            let em_count = (width / 2).max(20);
            let divider = "\u{2014}".repeat(em_count);
            out.push_str(&divider);
            out.push('\n');
            continue;
        }

        if let Some(prefix_len) = heading_prefix_len(line) {
            let wrapped = wrap_heading(line, prefix_len, width);
            out.push_str(&wrapped);
            out.push('\n');
            continue;
        }

        let wrapped = wrap_line(line, width);
        out.push_str(&wrapped);
        out.push('\n');
    }

    out
}

/// True if the line is a thematic-break marker: only dashes (≥3), possibly
/// with leading whitespace, nothing else.
fn is_divider_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && trimmed.len() >= 3
        && trimmed.chars().all(|c| c == '-')
}

/// Number of leading `#` chars plus the following space if the line is an
/// ATX heading (`# `, `## `, …). Otherwise `None`.
fn heading_prefix_len(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let hash_count = trimmed.chars().take_while(|c| *c == '#').count();
    if hash_count == 0 {
        return None;
    }
    let mut chars = trimmed.chars();
    for _ in 0..hash_count {
        chars.next();
    }
    if chars.next() == Some(' ') {
        let leading_ws = line.len() - trimmed.len();
        Some(leading_ws + hash_count + 1)
    } else {
        None
    }
}

/// Wrap an ATX heading line so the heading's text body fits inside `width`.
/// The `# ` prefix is kept on the first output line; continuation lines are
/// indented under the heading text so they visually belong to the heading
/// even though pdf_oxide renders every source line as its own PDF line.
/// Overlong single tokens hard-break via `wrap_overlong_token`.
fn wrap_heading(line: &str, prefix_len: usize, width: usize) -> String {
    let text = &line[prefix_len.min(line.len())..];
    let text = text.trim();
    if text.chars().count() <= width.saturating_sub(prefix_len) {
        return line.to_string();
    }

    let indent = " ".repeat(prefix_len);
    let budget = width.saturating_sub(prefix_len);
    if budget == 0 {
        return line.to_string();
    }

    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut result = String::new();
    let mut current = String::new();
    let mut current_chars = 0usize;
    let mut first_line = true;

    let flush_current = |result: &mut String,
                         current: &mut String,
                         current_chars: &mut usize,
                         first_line: &mut bool| {
        if current.is_empty() {
            return;
        }
        if *first_line {
            result.push_str(&line[..prefix_len]);
            *first_line = false;
        } else {
            result.push_str(&indent);
        }
        result.push_str(current);
        result.push('\n');
        current.clear();
        *current_chars = 0;
    };

    for token in tokens {
        let token_chars = token.chars().count();
        if token_chars > budget {
            flush_current(
                &mut result,
                &mut current,
                &mut current_chars,
                &mut first_line,
            );
            let sub = wrap_overlong_token(token, budget);
            for sub_line in sub.lines() {
                if first_line {
                    result.push_str(&line[..prefix_len]);
                    first_line = false;
                } else {
                    result.push_str(&indent);
                }
                result.push_str(sub_line);
                result.push('\n');
            }
        } else if current_chars == 0 {
            current = token.to_string();
            current_chars = token_chars;
        } else if current_chars + 1 + token_chars <= budget {
            current.push(' ');
            current.push_str(token);
            current_chars += 1 + token_chars;
        } else {
            flush_current(
                &mut result,
                &mut current,
                &mut current_chars,
                &mut first_line,
            );
            current = token.to_string();
            current_chars = token_chars;
        }
    }

    if !current.is_empty() {
        if first_line {
            result.push_str(&line[..prefix_len]);
        } else {
            result.push_str(&indent);
        }
        result.push_str(&current);
    }

    result.trim_end_matches('\n').to_string()
}

/// Leading whitespace plus optional list/blockquote marker width in bytes.
/// Continuation lines are indented with this many spaces so wrapped bullets
/// stay visually nested under the marker.
fn list_or_indent_prefix_len(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    let rest = &line[i..];
    if rest.starts_with("> ") {
        return i + 2;
    }
    if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
        return i + 2;
    }
    let digit_count = rest.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_count > 0 {
        let after = &rest[digit_count..];
        if after.starts_with(". ") || after.starts_with(") ") {
            return i + digit_count + 2;
        }
    }
    i
}

fn wrap_line(line: &str, width: usize) -> String {
    if line.chars().count() <= width {
        return line.to_string();
    }

    let prefix_len = list_or_indent_prefix_len(line).min(line.len());
    let prefix = &line[..prefix_len];
    let body = &line[prefix_len..];
    let prefix_chars = prefix.chars().count();

    let (first_prefix, cont_indent, budget) = if prefix_chars > 0 && prefix_chars < width {
        (
            prefix,
            " ".repeat(prefix_chars),
            width - prefix_chars,
        )
    } else {
        ("", String::new(), width)
    };

    let tokens: Vec<&str> = body.split_whitespace().collect();
    if tokens.is_empty() {
        return line.to_string();
    }

    let mut result = String::new();
    let mut current = String::new();
    let mut current_chars = 0usize;
    let mut first_line = true;

    let flush_current = |result: &mut String,
                         current: &mut String,
                         current_chars: &mut usize,
                         first_line: &mut bool| {
        if current.is_empty() {
            return;
        }
        if *first_line {
            result.push_str(first_prefix);
            *first_line = false;
        } else {
            result.push_str(&cont_indent);
        }
        result.push_str(current);
        result.push('\n');
        current.clear();
        *current_chars = 0;
    };

    for token in tokens {
        let token_chars = token.chars().count();
        if token_chars > budget {
            flush_current(
                &mut result,
                &mut current,
                &mut current_chars,
                &mut first_line,
            );
            let sub = wrap_overlong_token(token, budget);
            for sub_line in sub.lines() {
                if first_line {
                    result.push_str(first_prefix);
                    first_line = false;
                } else {
                    result.push_str(&cont_indent);
                }
                result.push_str(sub_line);
                result.push('\n');
            }
        } else if current_chars == 0 {
            current = token.to_string();
            current_chars = token_chars;
        } else if current_chars + 1 + token_chars <= budget {
            current.push(' ');
            current.push_str(token);
            current_chars += 1 + token_chars;
        } else {
            flush_current(
                &mut result,
                &mut current,
                &mut current_chars,
                &mut first_line,
            );
            current = token.to_string();
            current_chars = token_chars;
        }
    }

    if !current.is_empty() {
        if first_line {
            result.push_str(first_prefix);
        } else {
            result.push_str(&cont_indent);
        }
        result.push_str(&current);
    }

    result.trim_end_matches('\n').to_string()
}

fn wrap_overlong_token(token: &str, width: usize) -> String {
    let separators = ['/', '?', '&', '='];
    let mut parts: Vec<String> = Vec::new();
    let mut buf = String::new();
    for ch in token.chars() {
        buf.push(ch);
        if separators.contains(&ch) {
            parts.push(std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        parts.push(buf);
    }

    let mut result = String::new();
    let mut current = String::new();
    let mut current_chars = 0usize;

    for part in parts {
        let part_chars = part.chars().count();
        if part_chars > width {
            if current_chars > 0 {
                result.push_str(&current);
                result.push('\n');
                current.clear();
                current_chars = 0;
            }
            let chunked = hard_chunk(&part, width);
            result.push_str(&chunked);
            result.push('\n');
        } else if current_chars == 0 {
            current = part;
            current_chars = part_chars;
        } else if current_chars + part_chars <= width {
            current.push_str(&part);
            current_chars += part_chars;
        } else {
            result.push_str(&current);
            result.push('\n');
            current = part;
            current_chars = part_chars;
        }
    }

    if current_chars > 0 {
        result.push_str(&current);
    }

    result.trim_end_matches('\n').to_string()
}

fn hard_chunk(s: &str, width: usize) -> String {
    let indices: Vec<(usize, char)> = s.char_indices().collect();
    let total = indices.len();
    let mut result = String::new();
    let mut pos = 0;
    while pos < total {
        let end = (pos + width).min(total);
        let byte_start = indices[pos].0;
        let byte_end = if end < total { indices[end].0 } else { s.len() };
        result.push_str(&s[byte_start..byte_end]);
        result.push('\n');
        pos = end;
    }
    result.trim_end_matches('\n').to_string()
}
