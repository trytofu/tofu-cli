use reqwest::StatusCode;

#[derive(Debug)]
pub enum HealthStatus {
    Ok,
    NotOk(StatusCode)
}
