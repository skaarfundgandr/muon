pub fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    if let Some(inner) = (if lower.starts_with("```json") {
        trimmed.strip_prefix(&trimmed[.."```json".len()])
    } else if lower.starts_with("```") {
        trimmed.strip_prefix("```")
    } else {
        None
    })
    .and_then(|s| s.strip_suffix("```"))
    .map(str::trim)
    {
        return Some(inner);
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    Some(&trimmed[start..=end])
}
