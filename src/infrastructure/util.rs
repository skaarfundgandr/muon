use std::path::PathBuf;

pub fn expand_tilde<P: Into<PathBuf>>(p: P) -> PathBuf {
    let path = p.into();
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix('~')
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest.trim_start_matches('/'));
    }
    path
}

/// Extract the first JSON object from text that may contain markdown code fences
/// or surrounding prose.
pub fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    // Strip markdown code fences
    if let Some(inner) = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
    {
        return Some(inner);
    }
    // Find first '{' to last '}' as fallback
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    Some(&trimmed[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_extract_json_invalid() {
        assert_eq!(extract_json("no json here"), None);
        assert_eq!(extract_json(""), None);
    }
}
