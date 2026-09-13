use reqwest::{header, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::config::Config;

/// Thin, safe Porkbun v3 REST client. No secrets logged.
#[derive(Clone, Debug)]
pub struct PorkbunClient {
    client: Client,
    config: Config,
}

/// Stable error vocabulary matching Porkbun `code` values.
#[derive(Debug, thiserror::Error)]
pub enum PorkbunError {
    #[error("authentication failed: {message}")]
    Auth { message: String, retryable: bool },
    #[error("request invalid: {message} (code={code})")]
    InvalidRequest { message: String, code: String, retryable: bool },
    #[error("domain not found: {message}")]
    DomainNotFound { message: String },
    #[error("rate limited; retry after {after}s")]
    RateLimited { after: u64, message: String },
    #[error("insufficient funds or spend limit exceeded: {message}")]
    Funds { message: String, retryable: bool },
    #[error("network/transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("unknown error: status={status}, message={message}")]
    Unknown { status: u16, message: String, code: Option<String> },
}

impl PorkbunError {
    pub fn retryable(&self) -> bool {
        match self {
            PorkbunError::RateLimited { after, .. } => true,
            PorkbunError::Transport(_) => true,
            PorkbunError::Auth { retryable, .. } => *retryable,
            PorkbunError::Funds { retryable, .. } => *retryable,
            PorkbunError::InvalidRequest { retryable, .. } => *retryable,
            _ => false,
        }
    }
}

/// Canonical response wrapper from Porkbun v3.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PorkbunResponse<T = serde_json::Value> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<serde_json::Value>,
    // Payload payload varies; we deserialize generically and provide helpers.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub payload: Option<T>,
}

impl PorkbunClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(45))
            .default_headers({
                let mut h = header::HeaderMap::new();
                h.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
                h.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));
                h.insert(header::USER_AGENT, header::HeaderValue::from_static("porkbun-mcp/0.1.0 (Rust; rmcp 3.x; +https://github.com/atheerium/porkbun-mcp)"));
                h
            })
            .build()
            .expect("valid reqwest Client");
        PorkbunClient { client, config }
    }

    /// Build a unique idempotency key for mutation POSTs.
    pub fn idempotency_key() -> String {
        format!("pbmcp-{}", uuid::Uuid::now_v7())
    }

    /// General POST helper. Does NOT enforce keys for public endpoints (pricing/get with no auth).
    pub async fn post<B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
        idempotency: Option<&str>,
    ) -> Result<serde_json::Value, PorkbunError> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches("/"), path);
        let mut req = self.client.post(&url)
            .json(body);
        if self.config.has_keys() {
            req = req.header("X-API-Key", self.config.api_key.as_str())
                .header("X-Secret-API-Key", self.config.secret_api_key.as_str());
        } else if !self.config.docs_only {
            // For docs-only or mock-only, omit auth headers (pricing/get accepts no auth; mock requires none).
        }
        if let Some(key) = idempotency {
            req = req.header("Idempotency-Key", key);
        }

        debug!(url = %url, "Porkbun POST (path={})", path);
        let resp = req.send().await?;
        let status = resp.status();
        let headers = resp.headers().clone();

        // Handle rate limit header before consuming body.
        let retry_after = headers.get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        if status == StatusCode::TOO_MANY_REQUESTS || retry_after > 0 {
            let message = format!("rate limited; retry after {}s", retry_after);
            return Err(PorkbunError::RateLimited { after: retry_after, message });
        }

        let body_text: String = resp.text().await.unwrap_or_default();
        if body_text.is_empty() {
            return Err(PorkbunError::Unknown {
                status: status.as_u16(),
                message: "empty response body".into(),
                code: None,
            });
        }

        let parsed: serde_json::Value = serde_json::from_str(&body_text)
            .unwrap_or(serde_json::Value::String(body_text.clone()));

        // Detect Porkbun-level SUCCESS / ERROR.
        let status_str = parsed.get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        let message = parsed.get("message")
            .and_then(|m| m.as_str())
            .map(String::from);
        let code = parsed.get("code")
            .and_then(|c| c.as_str())
            .map(String::from);

        if status_str == "ERROR" || status.as_u16() >= 400 {
            // Classify by code if present.
            if let Some(ref c) = code {
                let retryable = match c.as_str() {
                    "RATE_LIMIT_EXCEEDED" => true,
                    "INSUFFICIENT_FUNDS" => false,
                    "MONTHLY_SPEND_LIMIT_EXCEEDED" => false,
                    "IDEMPOTENCY_KEY_MISMATCH" | "IDEMPOTENCY_KEY_IN_USE" => false,
                    "INVALID_AUTH_CODE" | "INVALID_DOMAIN" | "DOMAIN_NOT_FOUND" => false,
                    _ => true,
                };
                if c == "RATE_LIMIT_EXCEEDED" {
                    return Err(PorkbunError::RateLimited { after: retry_after, message: message.unwrap_or_default() });
                }
                if c == "INSUFFICIENT_FUNDS" || c == "MONTHLY_SPEND_LIMIT_EXCEEDED" {
                    return Err(PorkbunError::Funds { message: message.unwrap_or_default(), retryable });
                }
                return Err(PorkbunError::InvalidRequest {
                    message: message.unwrap_or_default(),
                    code: c.clone(),
                    retryable,
                });
            }
            return Err(PorkbunError::Unknown {
                status: status.as_u16(),
                message: message.unwrap_or_else(|| format!("HTTP {}", status.as_u16())),
                code,
            });
        }

        Ok(parsed)
    }

    /// GET helper (used by read endpoints supporting header auth).
    pub async fn get(&self, path: &str) -> Result<serde_json::Value, PorkbunError> {
        let url = format!("{}{}", self.config.base_url.trim_end_matches("/"), path);
        let mut req = self.client.get(&url);
        if self.config.has_keys() {
            req = req.header("X-API-Key", self.config.api_key.as_str())
                .header("X-Secret-API-Key", self.config.secret_api_key.as_str());
        }
        req = req.header("User-Agent", "porkbun-mcp/0.1.0 (Rust; rmcp 3.x)");
        debug!(url = %url, "Porkbun GET (path={})", path);
        let resp = req.send().await?;
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        let parsed: serde_json::Value = serde_json::from_str(&body_text)
            .unwrap_or(serde_json::Value::String(body_text));
        if status.as_u16() >= 400 {
            let message = parsed.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
            let code = parsed.get("code").and_then(|c| c.as_str()).map(String::from);
            return Err(PorkbunError::Unknown { status: status.as_u16(), message, code });
        }
        Ok(parsed)
    }
}
