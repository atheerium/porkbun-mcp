use std::env;

/// Porkbun MCP server configuration — read from environment. Never logs secrets.
#[derive(Clone, Debug)]
pub struct Config {
    /// Public API key (`pk1_...`). Required for live operations.
    pub api_key: String,
    /// Secret API key (`sk1_...`). Required for live operations.
    pub secret_api_key: String,
    /// Base URL override (defaults to official v3 endpoint).
    pub base_url: String,
    /// Docs base host (default `https://porkbun.com`).
    pub docs_base: String,
    /// Sandbox mode flag (set `PORKBUN_SANDBOX=true` to use sandbox keys; does not change URL — sandbox uses same base URL but responds with `sandbox:true`).
    pub sandbox: bool,
    /// Whether docs-only mode is active (`PORKBUN_DOCS_ONLY=true` allows `docs/*`, `mock/*` without keys).
    pub docs_only: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let api_key = env::var("PORKBUN_API_KEY").unwrap_or_default();
        let secret_api_key = env::var("PORKBUN_SECRET_API_KEY").unwrap_or_default();
        let sandbox = env::var("PORKBUN_SANDBOX").unwrap_or_default().to_lowercase() == "true" ||
            env::var("PORKBUN_SANDBOX").unwrap_or_default() == "1";
        let docs_only = env::var("PORKBUN_DOCS_ONLY").unwrap_or_default().to_lowercase() == "true" ||
            env::var("PORKBUN_DOCS_ONLY").unwrap_or_default() == "1";

        let base_url = env::var("PORKBUN_BASE_URL").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://api.porkbun.com/api/json/v3".to_string());
        let docs_base = env::var("PORKBUN_DOCS_BASE").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "https://porkbun.com".to_string());

        // For live operations (anything except docs-only or pure mock calls), keys are required.
        // We don't block at init time because docs-only mode is a valid use case.
        Ok(Config {
            api_key,
            secret_api_key,
            base_url,
            docs_base,
            sandbox,
            docs_only,
        })
    }

    /// Verify keys present for live mutation / read calls.
    pub fn has_keys(&self) -> bool {
        !self.api_key.is_empty() && !self.secret_api_key.is_empty()
    }

    /// Check sandbox prefix pattern (public info only, safe to inspect).
    pub fn is_sandbox_key(&self) -> bool {
        self.api_key.starts_with("pk1_sb_") || self.secret_api_key.starts_with("sk1_sb_")
    }
}
