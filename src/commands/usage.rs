use crate::{
    api::ApiClient,
    models::billing_status::{BillingLimits, BillingStatus, BillingUsage},
    utils::{api_errors::exit_api_error, output, time::fmt_time},
};

pub async fn run(client: &ApiClient, json: bool) {
    match client.billing_status().await {
        Ok(s) => print_usage(&s, json),
        Err(e) => exit_api_error(e, "fetch usage", None),
    }
}

fn print_usage(status: &BillingStatus, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::json!({
                "plan": &status.plan,
                "status": &status.status,
                "current_period_start": &status.current_period_start,
                "current_period_end": &status.current_period_end,
                "cancel_at_period_end": status.cancel_at_period_end,
                "usage": &status.usage,
                "limits": &status.limits,
                "remaining": usage_remaining(&status.usage, &status.limits),
            })
        );
        return;
    }

    print_plan_summary(status);
    print_usage_table(status);
}

fn print_plan_summary(status: &BillingStatus) {
    let mut rows = vec![
        ("Plan", output::cell(title_case(&status.plan))),
        ("Status", output::status_cell(&status.status)),
    ];

    if let Some(period_end) = status.current_period_end.as_deref() {
        rows.push(("Current period ends", output::cell(fmt_time(period_end))));
    }

    if status.cancel_at_period_end {
        rows.push(("Cancels at period end", output::status_cell("yes")));
    }

    println!("{}", output::kv_table_cells(rows));
}

fn print_usage_table(status: &BillingStatus) {
    let mut table = output::data_table(&["Resource", "Used", "Limit", "Remaining"]);
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

    table.add_row(vec![
        output::cell("Payload retention"),
        output::cell("-"),
        output::cell(payload_retention_label(&status.plan)),
        output::cell("-"),
    ]);
    table.add_row(vec![
        output::cell("Event history"),
        output::cell("-"),
        output::cell(format!("{} days", status.limits.retention_days)),
        output::cell("-"),
    ]);
    println!("{table}");
}

fn usage_remaining(usage: &BillingUsage, limits: &BillingLimits) -> serde_json::Value {
    serde_json::json!({
        "hooks": remaining(usage.hooks, limits.hooks),
        "targets_per_hook": remaining(usage.targets_per_hook, limits.targets_per_hook),
        "events_this_month": remaining(usage.events_this_month, limits.events_this_month),
        "workspace_members": remaining(usage.workspace_members, limits.workspace_members),
    })
}

fn add_usage_row(table: &mut comfy_table::Table, label: &str, used: i64, limit: i64) {
    table.add_row(vec![
        output::cell(label),
        output::cell(used.to_string()),
        output::cell(limit.to_string()),
        output::cell(remaining(used, limit).to_string()),
    ]);
}

fn remaining(used: i64, limit: i64) -> i64 {
    limit.saturating_sub(used)
}

fn payload_retention_label(plan: &str) -> &'static str {
    if plan == "pro" || plan == "premium" {
        "7 days"
    } else {
        "24 hours"
    }
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
