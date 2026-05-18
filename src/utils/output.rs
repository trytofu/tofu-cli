use colored::Colorize;
use comfy_table::{
    Attribute, Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

use crate::{
    api::models::billing_status::{BillingLimits, BillingStatus, BillingUsage},
    utils::time::fmt_time,
};

pub fn success(s: impl std::fmt::Display) {
    let s = s.to_string();
    let output = s.bright_green().bold();
    println!("{}", output)
}

pub fn error(s: impl std::fmt::Display) {
    let s = s.to_string();
    eprintln!("{}", s.bright_red().bold())
}

pub fn warning(s: impl std::fmt::Display) {
    let s = s.to_string();
    println!("{}", s.bright_yellow().bold())
}

pub fn command(s: impl std::fmt::Display) {
    let s = s.to_string();
    println!("{}", s.bright_blue().bold())
}

pub fn info(s: impl std::fmt::Display) {
    let s = s.to_string();
    println!("{}", s.blue().bold())
}

// Table

pub fn kv_table(rows: Vec<(&str, String)>) -> Table {
    kv_table_cells(
        rows.into_iter()
            .map(|(key, value)| (key, Cell::new(value)))
            .collect(),
    )
}

pub fn kv_table_cells(rows: Vec<(&str, Cell)>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    for (key, value) in rows {
        table.add_row(vec![Cell::new(key).add_attribute(Attribute::Bold), value]);
    }

    table
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
        ("Plan", title_case(&status.plan)),
        ("Status", status.status.replace('_', " ")),
    ];
    if let Some(period_end) = status.current_period_end.as_deref() {
        rows.push(("Current period ends", fmt_time(period_end)));
    }
    if status.cancel_at_period_end {
        rows.push(("Cancels at period end", "yes".to_string()));
    }
    println!("{}", kv_table(rows));

    let mut t = data_table(&["Resource", "Used", "Limit", "Remaining"]);
    add_usage_row(&mut t, "Hooks", status.usage.hooks, status.limits.hooks);
    add_usage_row(
        &mut t,
        "Targets per hook",
        status.usage.targets_per_hook,
        status.limits.targets_per_hook,
    );
    add_usage_row(
        &mut t,
        "Events this month",
        status.usage.events_this_month,
        status.limits.events_this_month,
    );
    add_usage_row(
        &mut t,
        "Workspace members",
        status.usage.workspace_members,
        status.limits.workspace_members,
    );
    let payload_retention = if status.plan == "pro" || status.plan == "premium" {
        "7 days".to_string()
    } else {
        "24 hours".to_string()
    };
    t.add_row(vec![
        cell("Payload retention"),
        cell("-"),
        cell(payload_retention),
        cell("-"),
    ]);
    t.add_row(vec![
        cell("Event history"),
        cell("-"),
        cell(format!("{} days", status.limits.retention_days)),
        cell("-"),
    ]);
    println!("{t}");
}

fn usage_remaining(usage: &BillingUsage, limits: &BillingLimits) -> serde_json::Value {
    serde_json::json!({
        "hooks": remaining(usage.hooks, limits.hooks),
        "targets_per_hook": remaining(usage.targets_per_hook, limits.targets_per_hook),
        "events_this_month": remaining(usage.events_this_month, limits.events_this_month),
        "workspace_members": remaining(usage.workspace_members, limits.workspace_members),
    })
}

fn add_usage_row(t: &mut comfy_table::Table, label: &str, used: i64, limit: i64) {
    t.add_row(vec![
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

pub fn cell(value: impl Into<String>) -> Cell {
    Cell::new(value.into())
}

pub fn data_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(
        headers
            .iter()
            .map(|header| Cell::new(*header).add_attribute(Attribute::Bold))
            .collect::<Vec<_>>(),
    );
    table
}
