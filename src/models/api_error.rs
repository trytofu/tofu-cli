#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Request(#[from] reqwest::Error),
}
