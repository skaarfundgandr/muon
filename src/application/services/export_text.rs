/// If the first non-empty line is an ATX H1 (`# …` but not `##`),
/// remove that line and any immediately following blank line.
/// Unconditional — does not compare to a title string.
pub fn strip_leading_h1(summary: &str) -> String {
    let all_lines: Vec<&str> = summary.lines().collect();

    let first_non_empty = all_lines.iter().position(|l| !l.trim().is_empty());
    let first_idx = match first_non_empty {
        Some(i) => i,
        None => return String::new(),
    };

    let first = all_lines[first_idx].trim_start();
    let is_h1 = first.starts_with("# ") && !first.starts_with("## ");
    if !is_h1 {
        return summary.to_string();
    }

    let mut skip = first_idx + 1;

    if skip < all_lines.len() && all_lines[skip].trim().is_empty() {
        skip += 1;
    }

    if skip >= all_lines.len() {
        return String::new();
    }

    let mut result = String::new();
    for (i, line) in all_lines.iter().enumerate().skip(skip) {
        if i > skip {
            result.push('\n');
        }
        result.push_str(line);
    }
    result
}

/// Soft-wrap markdown source at `width` (character budget) for PDF rendering.
/// Skips fenced code blocks; preserves blank lines and headings unwrapped.
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

        if line.trim().is_empty() || is_heading(line) {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        let wrapped = wrap_line(line, width);
        out.push_str(&wrapped);
        out.push('\n');
    }

    out
}

fn is_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    let hash_count = trimmed.chars().take_while(|c| *c == '#').count();
    hash_count > 0 && trimmed.chars().nth(hash_count) == Some(' ')
}

fn wrap_line(line: &str, width: usize) -> String {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut current = String::new();
    let mut current_chars = 0usize;

    for token in tokens {
        let token_chars = token.chars().count();
        if token_chars <= width {
            if current_chars == 0 {
                current = token.to_string();
                current_chars = token_chars;
            } else if current_chars + 1 + token_chars <= width {
                current.push(' ');
                current.push_str(token);
                current_chars += 1 + token_chars;
            } else {
                result.push_str(&current);
                result.push('\n');
                current = token.to_string();
                current_chars = token_chars;
            }
        } else {
            if current_chars > 0 {
                result.push_str(&current);
                result.push('\n');
                current.clear();
                current_chars = 0;
            }
            let sub = wrap_overlong_token(token, width);
            result.push_str(&sub);
            result.push('\n');
        }
    }

    if current_chars > 0 {
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
