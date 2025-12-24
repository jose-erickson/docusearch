use std::env;

/// Configuration for the SuccessFactors microservice
#[derive(Debug, Clone)]
pub struct Config {
    /// SuccessFactors API base URL (e.g., https://api.successfactors.com)
    pub sf_api_url: String,
    /// SuccessFactors company ID
    pub sf_company_id: String,
    /// SuccessFactors username
    pub sf_username: String,
    /// SuccessFactors password
    pub sf_password: String,
    /// Server host
    pub server_host: String,
    /// Server port
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            sf_api_url: env::var("SF_API_URL")
                .map_err(|_| ConfigError::MissingEnvVar("SF_API_URL"))?,
            sf_company_id: env::var("SF_COMPANY_ID")
                .map_err(|_| ConfigError::MissingEnvVar("SF_COMPANY_ID"))?,
            sf_username: env::var("SF_USERNAME")
                .map_err(|_| ConfigError::MissingEnvVar("SF_USERNAME"))?,
            sf_password: env::var("SF_PASSWORD")
                .map_err(|_| ConfigError::MissingEnvVar("SF_PASSWORD"))?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
        })
    }

    /// Get the full authentication string for SuccessFactors
    /// Format: username@companyId
    pub fn auth_username(&self) -> String {
        format!("{}@{}", self.sf_username, self.sf_company_id)
    }

    /// Get the server address
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("Invalid port number")]
    InvalidPort,
}
