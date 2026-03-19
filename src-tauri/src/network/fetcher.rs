use crate::error::NetworkError;
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn};

const FETCH_TIMEOUT_SECS: u64 = 30;
const MAX_REDIRECTS: usize = 5;
const MAX_RESPONSE_BYTES: usize = 50 * 1024 * 1024; // 50 MB hard cap per resource

pub struct ResourceResponse {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

pub struct ResourceFetcher {
    client: Client,
}

impl ResourceFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .user_agent(concat!("Postail/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_else(|e| {
                warn!("resource fetcher: failed to build client with config: {e}, falling back to default");
                Client::new()
            });

        Self { client }
    }

    pub async fn fetch(&self, url: &str) -> Result<ResourceResponse, NetworkError> {
        info!("resource fetch: url={url}");

        let response = self.client.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            warn!("resource fetch: url={url} status={status}");
            return Err(NetworkError::BadStatus {
                status: status.as_u16(),
                url: url.to_string(),
            });
        }

        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .map(|v| v.trim().to_lowercase())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let content_length = response.content_length();
        let bytes = response.bytes().await?;

        if bytes.len() > MAX_RESPONSE_BYTES {
            warn!(
                "resource fetch: url={url} size={} exceeds per-resource limit",
                bytes.len()
            );
            return Err(NetworkError::TooLarge {
                url: url.to_string(),
                size: bytes.len(),
            });
        }

        info!(
            "resource fetch: url={url} mime={mime_type} size={} declared={:?}",
            bytes.len(),
            content_length,
        );

        Ok(ResourceResponse {
            bytes: bytes.to_vec(),
            mime_type,
        })
    }
}
