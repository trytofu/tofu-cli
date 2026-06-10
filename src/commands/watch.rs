use futures_util::StreamExt;

use crate::{
    api::ApiClient,
    models::{api_error::ApiError, sse_message::SseMessage},
    utils::{
        output::{self, Tone, print_plan_limit_error},
        time::{current_clock_time, fmt_clock_time},
        workspace::resolve_hook_or_exit,
    },
};

pub async fn run(
    client: &ApiClient,
    slug: String,
    deliveries: bool,
    target: Option<String>,
    json: bool,
) {
    let hook = resolve_hook_or_exit(client, &slug).await;

    if !json {
        match client.get_workspace(&hook.workspace_id).await {
            Ok(w) => output::success(format!("Watching {} in workspace {}", hook.slug, w.slug)),
            Err(_) => output::success(format!("Watching {}", hook.slug)),
        }
        output::next_step(format!("Provider URL: {}", output::url(&hook.provider_url)));
    }

    let target_filter = target.as_deref();

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            if !json {
                output::warning("\nStopped watching.");
            }
        }

        _ = async {
            loop {
                match watch(client, &hook.id, deliveries, target_filter, json).await {
                    Ok(()) => {
                        // Stream ended, reconnect after a short delay
                    }
                    Err(ApiError::NotAuthenticated) => {
                        output::error("Not authenticated.");
                        output::warning(format!("Run {}.", output::command("tofu login")));
                        std::process::exit(1);
                    }
                    Err(ApiError::UnexpectedStatus { status })
                        if status == reqwest::StatusCode::UNAUTHORIZED =>
                    {
                        output::error("Invalid token.");
                        output::warning(format!(
                            "Run {} to re-authenticate.",
                            output::command("tofu login")
                        ));
                        std::process::exit(1);
                    }
                    Err(ApiError::UnexpectedStatus {status})
                        if status == reqwest::StatusCode::NOT_FOUND =>
                    {
                        output::error("Hook not found or you do not have access.");
                        std::process::exit(1);
                    }
                    Err(ApiError::UnexpectedStatus {status}) => {
                        if !json {
                            output::warning(format!("Server error: {status}. Reconnecting..."));
                        }
                    }
                    Err(ApiError::PlanLimitReached(err)) => {
                        if !json {
                            print_plan_limit_error(&err);
                        }
                        std::process::exit(1);
                    }
                    Err(ApiError::PayloadExpired(message)) => {
                        if !json {
                            output::warning(message);
                        }
                    }
                    Err(ApiError::Request(e)) => {
                        if !json {
                            output::warning(format!("Connection error: {e}. Reconnecting..."));
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        } => {}
    }
}

async fn watch(
    client: &ApiClient,
    hook_id: &str,
    deliveries: bool,
    target_filter: Option<&str>,
    json: bool,
) -> Result<(), ApiError> {
    let r = client.stream_events(hook_id).await?;
    let mut stream = r.bytes_stream();
    let mut buff = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(b) => {
                let text = String::from_utf8_lossy(&b);
                for event_data in parse_sse_events(&mut buff, &text) {
                    match serde_json::from_str::<SseMessage>(&event_data) {
                        Ok(SseMessage::EventReceived { event }) => {
                            if json {
                                println!("{event_data}");
                                continue;
                            }
                            let time = fmt_clock_time(&event.received_at);
                            let short_id = short_id(&event.id);
                            println!(
                                "{}  {}  {:<7}  {}",
                                output::paint(time, Tone::Muted),
                                short_id,
                                event.method,
                                output::paint("received", Tone::Success)
                            );
                        }
                        Ok(SseMessage::DeliveryCompleted { delivery }) => {
                            if !deliveries {
                                continue;
                            }

                            let matches_target =
                                target_filter.is_none_or(|f| delivery.target_name == f);

                            if !matches_target {
                                continue;
                            }

                            if json {
                                println!("{event_data}");
                                continue;
                            }

                            let time = current_clock_time();
                            let short_id = short_id(&delivery.event_id);

                            if delivery.status == "success" {
                                let status = delivery.response_status.unwrap_or(0);
                                let ms = delivery.duration_ms.unwrap_or(0);
                                println!(
                                    "{}  {}  -> {:<15} {}  {}ms",
                                    output::paint(time, Tone::Muted),
                                    short_id,
                                    delivery.target_name,
                                    output::paint(format!("HTTP {status}"), Tone::Success),
                                    ms
                                );
                            } else {
                                let ms = delivery.duration_ms.unwrap_or(0);
                                println!(
                                    "{}  {}  -> {:<15} {}  {}ms",
                                    output::paint(time, Tone::Muted),
                                    short_id,
                                    delivery.target_name,
                                    output::paint("failed", Tone::Error),
                                    ms
                                );
                            }
                        }
                        Err(_) => {
                            // Ignore malformed events
                        }
                    }
                }
            }
            Err(e) => {
                output::error(format!("Stream error: {e}"));
                return Ok(());
            }
        }
    }

    Ok(())
}

fn parse_sse_events(buffer: &mut String, chunk: &str) -> Vec<String> {
    buffer.push_str(chunk);
    let mut events = Vec::new();

    while let Some(pos) = buffer.find("\n\n") {
        let event_text = buffer[..pos].to_string();
        buffer.replace_range(..pos + 2, "");

        let mut data = String::new();
        for line in event_text.lines() {
            if let Some(rest) = line.strip_prefix("data:") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(rest.trim_start());
            }
        }
        if !data.is_empty() {
            events.push(data);
        }
    }

    events
}

fn short_id(uuid: &str) -> &str {
    uuid.get(..8).unwrap_or(uuid)
}
