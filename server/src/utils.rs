/// Truncate a string at a safe UTF-8 char boundary, appending "…" if truncated.
pub(crate) fn truncate_at_char_boundary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut boundary = max_len;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        format!("{}…", &s[..boundary])
    }
}
