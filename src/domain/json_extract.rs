pub fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if let Some(inner) = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(str::trim)
    {
        return Some(inner);
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    Some(&trimmed[start..=end])
}
