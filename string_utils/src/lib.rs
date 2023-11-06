pub fn strip_whitespace(s: &str) -> Option<&str> {
    let stripped_prefix = s.strip_prefix(" ")
        .or(Some(s))
        .unwrap();
    let stripped_suffix = stripped_prefix
        .strip_suffix(" ")
        .or(Some(stripped_prefix))
        .unwrap();
    if stripped_suffix.len() == 0 {
        None
    } else {
        Some(stripped_suffix)
    }
}

pub fn normalize_quotes(s: &str) -> String {
    s.replace("\"", "")
}
