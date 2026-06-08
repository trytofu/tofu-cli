use chrono::{DateTime, Utc};

pub fn fmt_time(iso: &str) -> String {
    DateTime::parse_from_rfc3339(iso).map_or_else(
        |_| iso.get(..19).unwrap_or(iso).replace('T', " "),
        |dt| {
            dt.with_timezone(&Utc)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        },
    )
}

pub fn fmt_clock_time(iso: &str) -> String {
    DateTime::parse_from_rfc3339(iso).map_or_else(
        |_| "??:??:??".to_string(),
        |dt| dt.with_timezone(&Utc).format("%H:%M:%S").to_string(),
    )
}

pub fn current_clock_time() -> String {
    Utc::now().format("%H:%M:%S").to_string()
}
