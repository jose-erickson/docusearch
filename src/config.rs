use secrecy::{ExposeSecret, SecretString};
use std::env;
use url::Url;

/// Maximum records per request enforced at the API boundary
pub const MAX_PAGE_SIZE: u32 = 500;
/// Default page size when client doesn't specify
pub const DEFAULT_PAGE_SIZE: u32 = 50;
/// Maximum skip value to prevent excessively deep pagination
pub const MAX_SKIP: u32 = 100_000;

/// Configuration for the SuccessFactors microservice.
///
/// Secrets (password, API key) are wrapped in `SecretString` which
/// zeroes the memory on drop and prevents accidental printing via Debug.
#[derive(Clone)]
pub struct Config {
    /// SuccessFactors API base URL (HTTPS enforced)
    pub sf_api_url: Url,
    /// SuccessFactors company ID
    pub sf_company_id: String,
    /// SuccessFactors username
    pub sf_username: String,
    /// SuccessFactors password (zeroed on drop)
    pub sf_password: SecretString,
    /// API key required to call this microservice (zeroed on drop)
    pub service_api_key: SecretString,
    /// Server host
    pub server_host: String,
    /// Server port
    pub server_port: u16,
    /// Comma-separated list of allowed CORS origins (empty = none)
    pub cors_allowed_origins: Vec<String>,
    /// Requests per second allowed per client IP
    pub rate_limit_rps: u32,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum request body size in bytes
    pub max_body_bytes: usize,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("sf_api_url", &self.sf_api_url.as_str())
            .field("sf_company_id", &self.sf_company_id)
            .field("sf_username", &self.sf_username)
            .field("sf_password", &"***REDACTED***")
            .field("service_api_key", &"***REDACTED***")
            .field("server_host", &self.server_host)
            .field("server_port", &self.server_port)
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("rate_limit_rps", &self.rate_limit_rps)
            .field("request_timeout_secs", &self.request_timeout_secs)
            .field("max_body_bytes", &self.max_body_bytes)
            .finish()
    }
}

impl Config {
    /// Load configuration from environment variables with validation.
    pub fn from_env() -> Result<Self, ConfigError> {
        let sf_api_url_raw = required_env("SF_API_URL")?;
        let sf_api_url = Url::parse(&sf_api_url_raw)
            .map_err(|_| ConfigError::InvalidValue("SF_API_URL", "not a valid URL"))?;

        // Security: enforce HTTPS for production (allow http only for localhost)
        if sf_api_url.scheme() != "https" {
            let host = sf_api_url.host_str().unwrap_or("");
            let is_local = matches!(host, "localhost" | "127.0.0.1" | "::1");
            if !is_local {
                return Err(ConfigError::InvalidValue(
                    "SF_API_URL",
                    "must use HTTPS (http allowed only for localhost)",
                ));
            }
        }

        let sf_company_id = required_env("SF_COMPANY_ID")?;
        let sf_username = required_env("SF_USERNAME")?;
        let sf_password = SecretString::new(required_env("SF_PASSWORD")?);

        let service_api_key_raw = required_env("SERVICE_API_KEY")?;
        // Security: enforce minimum length for the service API key
        if service_api_key_raw.len() < 32 {
            return Err(ConfigError::InvalidValue(
                "SERVICE_API_KEY",
                "must be at least 32 characters (use a cryptographically random value)",
            ));
        }
        let service_api_key = SecretString::new(service_api_key_raw);

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = parse_env("SERVER_PORT", 3000)?;

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        let rate_limit_rps = parse_env("RATE_LIMIT_RPS", 10u32)?;
        let request_timeout_secs = parse_env("REQUEST_TIMEOUT_SECS", 30u64)?;
        let max_body_bytes = parse_env("MAX_BODY_BYTES", 16_384usize)?;

        Ok(Self {
            sf_api_url,
            sf_company_id,
            sf_username,
            sf_password,
            service_api_key,
            server_host,
            server_port,
            cors_allowed_origins,
            rate_limit_rps,
            request_timeout_secs,
            max_body_bytes,
        })
    }

    /// Returns the composite username SuccessFactors expects: `user@company`.
    pub fn auth_username(&self) -> String {
        format!("{}@{}", self.sf_username, self.sf_company_id)
    }

    /// Exposes the password only for building the auth header.
    pub fn password(&self) -> &str {
        self.sf_password.expose_secret()
    }

    /// Exposes the API key only for constant-time comparison.
    pub fn api_key(&self) -> &str {
        self.service_api_key.expose_secret()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

fn required_env(key: &'static str) -> Result<String, ConfigError> {
    let value = env::var(key).map_err(|_| ConfigError::MissingEnvVar(key))?;
    if value.trim().is_empty() {
        return Err(ConfigError::InvalidValue(key, "must not be empty"));
    }
    Ok(value)
}

fn parse_env<T>(key: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    match env::var(key) {
        Ok(v) => v
            .parse::<T>()
            .map_err(|_| ConfigError::InvalidValue(key, "could not be parsed")),
        Err(_) => Ok(default),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("invalid value for {0}: {1}")]
    InvalidValue(&'static str, &'static str),
}
