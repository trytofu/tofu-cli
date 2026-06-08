use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SseMessage {
    #[serde(rename = "event.received")]
    EventReceived { event: EventPayload },
    #[serde(rename = "delivery.completed")]
    DeliveryCompleted { delivery: DeliveryPayload },
}

#[derive(Debug, Deserialize)]
pub struct EventPayload {
    pub id: String,
    pub method: String,
    pub received_at: String,
}

#[derive(Debug, Deserialize)]
pub struct DeliveryPayload {
    pub event_id: String,
    pub target_name: String,
    pub status: String,
    pub response_status: Option<i32>,
    pub duration_ms: Option<i32>,
}
