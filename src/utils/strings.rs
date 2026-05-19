
pub fn slugify_workspace_slug(value: &str) -> String {
    let mut s = String::new();

    for ch in value.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            s.push(ch);
        } else {
            s.push('-');
        }
    }

    s.trim_matches('-').to_string()
}
