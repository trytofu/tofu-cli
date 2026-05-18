pub fn fmt_time(iso: &str) -> String {
    iso.get(..19).unwrap_or(iso).replace('T', " ")
}
