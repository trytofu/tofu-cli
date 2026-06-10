use std::fmt::Write as _;

use crate::{
    api::ApiClient,
    models::events::{DeliveryDetail, EventDetail},
    utils::{
        api_errors::exit_api_error,
        output::{self, Tone},
        time::fmt_time,
        workspace::resolve_hook_or_exit,
    },
};

pub async fn list(client: &ApiClient, hook_slug: String, limit: u32, json: bool) {
    let hook = resolve_hook_or_exit(client, &hook_slug).await;

    match client.list_events(&hook.id, limit).await {
        Ok(events) => {
            if json {
                println!("{}", serde_json::json!({ "events": events }));
            } else if events.is_empty() {
                output::empty(format!(
                    "No events found for hook '{hook_slug}'. Send a webhook to the provider URL, then run this command again."
                ));
            } else {
                let mut t =
                    output::data_table(&["ID", "Time", "Method", "Path", "Replay", "Deliveries"]);
                for e in events {
                    let summary = &e.delivery_summary;
                    let mut status = format!(
                        "{} delivered ({} success, {} failed",
                        summary.total, summary.success, summary.failed
                    );
                    if summary.pending > 0 {
                        write!(status, ", {} pending", summary.pending)
                            .expect("writing to String cannot fail");
                    }
                    status.push(')');
                    let replay_status = if e.replay_available {
                        "available"
                    } else {
                        "expired"
                    };
                    t.add_row(vec![
                        output::cell(e.id),
                        output::cell(fmt_time(&e.received_at)),
                        output::cell(e.method),
                        output::cell(e.path),
                        output::status_cell(replay_status),
                        output::cell(status),
                    ]);
                }
                println!("{t}");
            }
        }
        Err(e) => exit_api_error(
            e,
            "list events",
            Some("Hook not found or you do not have access."),
        ),
    }
}

pub async fn show(client: &ApiClient, event_id: String, json: bool) {
    match client.get_event(&event_id).await {
        Ok(event) => print_event(event, json),
        Err(e) => exit_api_error(
            e,
            "fetch event",
            Some("Event not found or you do not have access."),
        ),
    }
}

pub async fn latest(client: &ApiClient, hook_slug: String, json: bool) {
    let hook = resolve_hook_or_exit(client, &hook_slug).await;

    let events = match client.list_events(&hook.id, 1).await {
        Ok(events) => events,
        Err(e) => exit_api_error(
            e,
            "list events",
            Some("Hook not found or you do not have access."),
        ),
    };

    let Some(event) = events.into_iter().next() else {
        if json {
            println!("{}", serde_json::json!({ "event": null }));
        } else {
            output::empty(format!("No events found for hook '{hook_slug}'."));
        }
        return;
    };

    match client.get_event(&event.id).await {
        Ok(event) => print_event(event, json),
        Err(e) => exit_api_error(
            e,
            "fetch event",
            Some("Event not found or you do not have access."),
        ),
    }
}

pub async fn expire(client: &ApiClient, event_id: String, json: bool) {
    let event = match client.get_event(&event_id).await {
        Ok(event) => event,
        Err(e) => exit_api_error(
            e,
            "fetch event",
            Some("Event not found or you do not have access."),
        ),
    };

    if event.payload_expired {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "already_expired": true,
                    "event": event,
                })
            );
        } else {
            output::empty(format!("Event payload was already expired: {event_id}"));
        }
        return;
    }

    match client.expire_event(&event_id).await {
        Ok(event) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "ok",
                        "already_expired": false,
                        "event": event,
                    })
                );
            } else {
                output::success(format!("Expired event payload: {event_id}"));
            }
        }
        Err(e) => exit_api_error(
            e,
            "expire event",
            Some("Event not found or you do not have access."),
        ),
    }
}

// helpers

fn print_event(event: EventDetail, json: bool) {
    if json {
        print_event_json(&event);
    } else {
        print_event_text(event);
    }
}

fn print_event_json(event: &EventDetail) {
    println!(
        "{}",
        serde_json::json!({
            "id": &event.id,
            "hook_id": &event.hook_id,
            "method": &event.method,
            "path": &event.path,
            "query_string": &event.query_string,
            "headers": &event.headers,
            "body_preview": &event.body_preview,
            "received_at": &event.received_at,
            "payload_expires_at": &event.payload_expires_at,
            "metadata_expires_at": &event.metadata_expires_at,
            "payload_expired_at": &event.payload_expired_at,
            "manually_expired_at": &event.manually_expired_at,
            "payload_expired": event.payload_expired,
            "replay_available": event.replay_available,
            "deliveries": &event.deliveries,
        })
    );
}

fn print_event_text(event: EventDetail) {
    let EventDetail {
        id,
        method,
        path,
        query_string,
        headers,
        body_preview,
        received_at,
        payload_expires_at,
        payload_expired,
        deliveries,
        ..
    } = event;

    let mut rows = vec![
        ("Event", id),
        ("Method", method),
        ("Path", path),
        ("Received", fmt_time(&received_at)),
    ];
    if let Some(qs) = query_string {
        rows.push(("Query", qs));
    }
    if payload_expired {
        rows.push(("Payload", output::paint("expired", Tone::Warning)));
        rows.push(("Replay", output::paint("unavailable", Tone::Warning)));
    } else {
        rows.push(("Payload expires", fmt_time(&payload_expires_at)));
        rows.push(("Replay", output::paint("available", Tone::Success)));
    }
    print_detail_rows(&rows);
    print_body_preview(body_preview);
    print_headers(&headers);
    print_deliveries(deliveries);
}

fn print_body_preview(body_preview: Option<String>) {
    if let Some(preview) = body_preview {
        println!("\n{}", output::paint("Body preview", Tone::Muted));
        println!("{}", fmt_body_preview(&preview));
    }
}

fn print_headers(headers: &serde_json::Value) {
    if let Some(headers) = headers.as_object()
        && !headers.is_empty()
    {
        let mut table = output::data_table(&["Header", "Value"]);
        for (key, value) in headers {
            table.add_row(vec![
                output::cell(key.to_string()),
                output::cell(value.to_string()),
            ]);
        }
        println!("\nHeaders:\n{table}");
    }
}

fn print_deliveries(deliveries: Vec<DeliveryDetail>) {
    if deliveries.is_empty() {
        output::empty("\nNo deliveries found for this event.");
        return;
    }

    let mut table = output::data_table(&["Attempted", "Target", "URL", "Result", "Duration"]);
    for delivery in deliveries {
        add_delivery_row(&mut table, delivery);
    }
    println!("\nDeliveries:\n{table}");
}

fn add_delivery_row(table: &mut comfy_table::Table, delivery: DeliveryDetail) {
    match delivery.status.as_str() {
        "success" => add_success_delivery_row(table, delivery),
        "failed" => add_failed_delivery_row(table, delivery),
        _ => {
            table.add_row(vec![
                output::cell(fmt_time(&delivery.attempted_at)),
                output::cell(delivery.target_name),
                output::url_cell(&delivery.target_url),
                output::status_cell(delivery.status),
                output::cell(""),
            ]);
        }
    }
}

fn add_success_delivery_row(table: &mut comfy_table::Table, delivery: DeliveryDetail) {
    let status = delivery.response_status.unwrap_or(0);
    let ms = delivery.duration_ms.unwrap_or(0);
    table.add_row(vec![
        output::cell(fmt_time(&delivery.attempted_at)),
        output::cell(delivery.target_name),
        output::url_cell(&delivery.target_url),
        output::tone_cell(format!("HTTP {status}"), Tone::Success),
        output::cell(format!("{ms}ms")),
    ]);
}

fn add_failed_delivery_row(table: &mut comfy_table::Table, delivery: DeliveryDetail) {
    let ms = delivery.duration_ms.unwrap_or(0);
    let reason = delivery.error_message.unwrap_or_default();
    let result = if reason.is_empty() {
        "failed".to_string()
    } else {
        format!("failed: {reason}")
    };
    table.add_row(vec![
        output::cell(fmt_time(&delivery.attempted_at)),
        output::cell(delivery.target_name),
        output::url_cell(&delivery.target_url),
        output::tone_cell(result, Tone::Error),
        output::cell(format!("{ms}ms")),
    ]);
}

fn print_detail_rows(rows: &[(&str, String)]) {
    let label_width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, value) in rows {
        println!("{label:<label_width$}  {value}");
    }
}

fn fmt_body_preview(preview: &str) -> String {
    serde_json::from_str::<serde_json::Value>(preview)
        .and_then(|value| serde_json::to_string_pretty(&value))
        .unwrap_or_else(|_| preview.to_string())
}
