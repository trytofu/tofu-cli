use std::io::IsTerminal;

use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

use crate::{
    models::billing_status::{BillingLimits, BillingStatus, BillingUsage},
    utils::time::fmt_time,
};

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy)]
pub enum Tone {
    Success,
    Warning,
    Error,
    Info,
    Muted,
}

#[derive(Clone, Copy)]
pub struct Theme {
    stream: Stream,
}

impl Theme {
    pub fn stdout() -> Self {
        Self {
            stream: Stream::Stdout,
        }
    }

    pub fn stderr() -> Self {
        Self {
            stream: Stream::Stderr,
        }
    }

    pub fn paint(self, value: impl AsRef<str>, tone: Tone) -> String {
        let value = value.as_ref();
        if color_enabled(self.stream) {
            format!("\x1b[{}m{}\x1b[0m", ansi_color(tone), value)
        } else {
            value.to_string()
        }
    }

    pub fn command(self, value: impl AsRef<str>) -> String {
        self.paint(value, Tone::Info)
    }

    pub fn url(self, value: impl AsRef<str>) -> String {
        self.paint(value, Tone::Info)
    }

    pub fn status_cell(self, status: impl AsRef<str>) -> Cell {
        let status = status.as_ref();
        match status.to_ascii_lowercase().as_str() {
            "available" | "enabled" | "healthy" | "ok" | "success" | "active" | "delivered" => {
                self.tone_cell(status, Tone::Success)
            }
            "disabled" | "expired" | "pending" | "unavailable" | "cancelled" => {
                self.tone_cell(status, Tone::Warning)
            }
            "failed" | "error" | "denied" => self.tone_cell(status, Tone::Error),
            _ => cell(status),
        }
    }

    pub fn tone_cell(self, value: impl Into<String>, tone: Tone) -> Cell {
        let cell = Cell::new(value.into());
        if stdout_color_enabled() {
            cell.fg(table_color(tone))
        } else {
            cell
        }
    }
}

pub fn theme() -> Theme {
    Theme::stdout()
}

pub fn stderr_theme() -> Theme {
    Theme::stderr()
}

pub fn paint(value: impl AsRef<str>, tone: Tone) -> String {
    theme().paint(value, tone)
}

pub fn command(value: impl AsRef<str>) -> String {
    theme().command(value)
}

pub fn url(value: impl AsRef<str>) -> String {
    theme().url(value)
}

pub fn success(message: impl AsRef<str>) {
    println!("{}", paint(message, Tone::Success));
}

pub fn warning(message: impl AsRef<str>) {
    eprintln!("{}", stderr_theme().paint(message, Tone::Warning));
}

pub fn error(message: impl AsRef<str>) {
    eprintln!("{}", stderr_theme().paint(message, Tone::Error));
}

#[allow(dead_code)]
pub fn info(message: impl AsRef<str>) {
    println!("{}", paint(message, Tone::Info));
}

#[allow(dead_code)]
pub fn empty(message: impl AsRef<str>) {
    println!("{}", paint(message, Tone::Muted));
}

pub fn next_step(message: impl AsRef<str>) {
    println!("{}", paint(message, Tone::Info));
}

pub fn data_table(headers: &[&str]) -> Table {
    let mut table = base_table();
    table.set_header(
        headers
            .iter()
            .map(|header| Cell::new(*header).add_attribute(Attribute::Bold))
            .collect::<Vec<_>>(),
    );
    table
}

pub fn kv_table(rows: Vec<(&str, String)>) -> Table {
    kv_table_cells(
        rows.into_iter()
            .map(|(key, value)| (key, Cell::new(value)))
            .collect(),
    )
}

pub fn kv_table_cells(rows: Vec<(&str, Cell)>) -> Table {
    let mut table = base_table();

    for (key, value) in rows {
        table.add_row(vec![Cell::new(key).add_attribute(Attribute::Bold), value]);
    }

    table
}

pub fn cell(value: impl Into<String>) -> Cell {
    Cell::new(value.into())
}

#[allow(dead_code)]
pub fn tone_cell(value: impl Into<String>, tone: Tone) -> Cell {
    theme().tone_cell(value, tone)
}

#[allow(dead_code)]
pub fn url_cell(value: impl AsRef<str>) -> Cell {
    tone_cell(value.as_ref(), Tone::Info)
}

pub fn status_cell(status: impl AsRef<str>) -> Cell {
    theme().status_cell(status)
}

pub fn print_usage(status: BillingStatus, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::json!({
                "plan": status.plan,
                "status": status.status,
                "current_period_start": status.current_period_start,
                "current_period_end": status.current_period_end,
                "cancel_at_period_end": status.cancel_at_period_end,
                "usage": status.usage,
                "limits": status.limits,
                "remaining": usage_remaining(&status.usage, &status.limits),
            })
        );
        return;
    }

    let mut rows = vec![
        ("Plan", cell(title_case(&status.plan))),
        ("Status", status_cell(&status.status)),
    ];
    if let Some(period_end) = status.current_period_end.as_deref() {
        rows.push(("Current period ends", cell(fmt_time(period_end))));
    }
    if status.cancel_at_period_end {
        rows.push(("Cancels at period end", status_cell("yes")));
    }
    println!("{}", kv_table_cells(rows));

    let mut table = data_table(&["Resource", "Used", "Limit", "Remaining"]);
    add_usage_row(&mut table, "Hooks", status.usage.hooks, status.limits.hooks);
    add_usage_row(
        &mut table,
        "Targets per hook",
        status.usage.targets_per_hook,
        status.limits.targets_per_hook,
    );
    add_usage_row(
        &mut table,
        "Events this month",
        status.usage.events_this_month,
        status.limits.events_this_month,
    );
    add_usage_row(
        &mut table,
        "Workspace members",
        status.usage.workspace_members,
        status.limits.workspace_members,
    );

    let payload_retention = if status.plan == "pro" || status.plan == "premium" {
        "7 days".to_string()
    } else {
        "24 hours".to_string()
    };
    table.add_row(vec![
        cell("Payload retention"),
        cell("-"),
        cell(payload_retention),
        cell("-"),
    ]);
    table.add_row(vec![
        cell("Event history"),
        cell("-"),
        cell(format!("{} days", status.limits.retention_days)),
        cell("-"),
    ]);
    println!("{table}");
}

fn base_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

pub fn stdout_color_enabled() -> bool {
    std::io::stdout().is_terminal() && !no_color()
}

fn stderr_color_enabled() -> bool {
    std::io::stderr().is_terminal() && !no_color()
}

fn color_enabled(stream: Stream) -> bool {
    match stream {
        Stream::Stdout => stdout_color_enabled(),
        Stream::Stderr => stderr_color_enabled(),
    }
}

fn ansi_color(tone: Tone) -> &'static str {
    match tone {
        Tone::Success => "32",
        Tone::Warning => "33",
        Tone::Error => "31",
        Tone::Info => "36",
        Tone::Muted => "90",
    }
}

fn table_color(tone: Tone) -> Color {
    match tone {
        Tone::Success => Color::Green,
        Tone::Warning => Color::Yellow,
        Tone::Error => Color::Red,
        Tone::Info => Color::Cyan,
        Tone::Muted => Color::DarkGrey,
    }
}

fn usage_remaining(usage: &BillingUsage, limits: &BillingLimits) -> serde_json::Value {
    serde_json::json!({
        "hooks": remaining(usage.hooks, limits.hooks),
        "targets_per_hook": remaining(usage.targets_per_hook, limits.targets_per_hook),
        "events_this_month": remaining(usage.events_this_month, limits.events_this_month),
        "workspace_members": remaining(usage.workspace_members, limits.workspace_members),
    })
}

fn add_usage_row(table: &mut Table, label: &str, used: i64, limit: i64) {
    table.add_row(vec![
        cell(label),
        cell(used.to_string()),
        cell(limit.to_string()),
        cell(remaining(used, limit).to_string()),
    ]);
}

fn remaining(used: i64, limit: i64) -> i64 {
    limit.saturating_sub(used)
}

fn title_case(value: &str) -> String {
    value
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
