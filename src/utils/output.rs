use std::io::IsTerminal;

use colored::{ColoredString, Colorize};
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

use crate::api::PlanLimitApiError;

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
    colors: bool,
}

impl Theme {
    pub fn stdout() -> Self {
        Self {
            colors: stdout_color_enabled(),
        }
    }

    pub fn stderr() -> Self {
        Self {
            colors: stderr_color_enabled(),
        }
    }

    pub fn paint(self, value: impl AsRef<str>, tone: Tone) -> String {
        let value = value.as_ref();
        if self.colors {
            style(value, tone).to_string()
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
        if self.colors {
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

pub fn tone_cell(value: impl Into<String>, tone: Tone) -> Cell {
    theme().tone_cell(value, tone)
}

pub fn url_cell(value: impl AsRef<str>) -> Cell {
    tone_cell(value.as_ref(), Tone::Info)
}

pub fn status_cell(status: impl AsRef<str>) -> Cell {
    theme().status_cell(status)
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
    terminal_color_enabled(std::io::stdout().is_terminal())
}

fn stderr_color_enabled() -> bool {
    terminal_color_enabled(std::io::stderr().is_terminal())
}

fn terminal_color_enabled(is_terminal: bool) -> bool {
    is_terminal && !no_color()
}

fn style(value: &str, tone: Tone) -> ColoredString {
    match tone {
        Tone::Success => value.green().bold(),
        Tone::Warning => value.yellow().bold(),
        Tone::Error => value.red().bold(),
        Tone::Info => value.cyan().bold(),
        Tone::Muted => value.dimmed(),
    }
}

fn table_color(tone: Tone) -> TableColor {
    match tone {
        Tone::Success => TableColor::Green,
        Tone::Warning => TableColor::Yellow,
        Tone::Error => TableColor::Red,
        Tone::Info => TableColor::Cyan,
        Tone::Muted => TableColor::DarkGrey,
    }
}

pub fn print_plan_limit_error(err: &PlanLimitApiError) {
    error(&err.message);
    warning(format!("Upgrade: {}", url(&err.upgrade_url)));
}
