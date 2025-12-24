use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::models::*;

/// SuccessFactors OData API client
#[derive(Clone)]
pub struct SuccessFactorsClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl SuccessFactorsClient {
    /// Create a new SuccessFactors client
    pub fn new(config: &Config) -> Result<Self, ClientError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ClientError::HttpClient(e.to_string()))?;

        // Create Basic Auth header
        let auth_string = format!("{}:{}", config.auth_username(), config.sf_password);
        let auth_header = format!("Basic {}", STANDARD.encode(auth_string));

        let base_url = config.sf_api_url.trim_end_matches('/').to_string();

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    /// Build the OData API URL
    fn build_url(&self, entity: &str, query: Option<&str>) -> String {
        let base = format!("{}/odata/v2/{}", self.base_url, entity);
        match query {
            Some(q) => format!("{}?{}", base, q),
            None => base,
        }
    }

    /// Make an authenticated GET request to the SuccessFactors API
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ClientError> {
        debug!("Making request to: {}", url);

        let response = self
            .client
            .get(url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ClientError::Request(e.to_string()))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("API error: {} - {}", status, error_text);
            return Err(ClientError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let body = response
            .json::<T>()
            .await
            .map_err(|e| ClientError::Parse(e.to_string()))?;

        Ok(body)
    }

    /// Get all users with pagination support
    pub async fn get_users(&self, top: Option<u32>, skip: Option<u32>) -> Result<Vec<User>, ClientError> {
        let mut query_parts = vec![
            "$select=userId,username,defaultFullName,email,status,hireDate".to_string(),
        ];

        if let Some(t) = top {
            query_parts.push(format!("$top={}", t));
        }
        if let Some(s) = skip {
            query_parts.push(format!("$skip={}", s));
        }

        let query = query_parts.join("&");
        let url = self.build_url("User", Some(&query));

        info!("Fetching users from SuccessFactors");
        let response: ODataResponse<User> = self.get(&url).await?;
        Ok(response.data.results)
    }

    /// Get a specific user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User, ClientError> {
        let url = format!(
            "{}/odata/v2/User('{}')?$select=userId,username,defaultFullName,email,status,hireDate",
            self.base_url, user_id
        );

        info!("Fetching user {} from SuccessFactors", user_id);
        let response: ODataSingleResponse<User> = self.get(&url).await?;
        Ok(response.data)
    }

    /// Get employee personal information
    pub async fn get_per_personal(&self, top: Option<u32>, skip: Option<u32>) -> Result<Vec<PerPersonal>, ClientError> {
        let mut query_parts = vec![
            "$select=personIdExternal,firstName,lastName,middleName,displayName,gender,nationality,dateOfBirth,maritalStatus".to_string(),
        ];

        if let Some(t) = top {
            query_parts.push(format!("$top={}", t));
        }
        if let Some(s) = skip {
            query_parts.push(format!("$skip={}", s));
        }

        let query = query_parts.join("&");
        let url = self.build_url("PerPersonal", Some(&query));

        info!("Fetching personal info from SuccessFactors");
        let response: ODataResponse<PerPersonal> = self.get(&url).await?;
        Ok(response.data.results)
    }

    /// Get employee job information
    pub async fn get_emp_job(&self, top: Option<u32>, skip: Option<u32>) -> Result<Vec<EmpJob>, ClientError> {
        let mut query_parts = vec![
            "$select=userId,jobCode,jobTitle,department,division,location,managerId,costCenter,startDate".to_string(),
        ];

        if let Some(t) = top {
            query_parts.push(format!("$top={}", t));
        }
        if let Some(s) = skip {
            query_parts.push(format!("$skip={}", s));
        }

        let query = query_parts.join("&");
        let url = self.build_url("EmpJob", Some(&query));

        info!("Fetching job info from SuccessFactors");
        let response: ODataResponse<EmpJob> = self.get(&url).await?;
        Ok(response.data.results)
    }

    /// Get employee employment information
    pub async fn get_emp_employment(&self, top: Option<u32>, skip: Option<u32>) -> Result<Vec<EmpEmployment>, ClientError> {
        let mut query_parts = vec![
            "$select=personIdExternal,userId,startDate,endDate,employmentStatus".to_string(),
        ];

        if let Some(t) = top {
            query_parts.push(format!("$top={}", t));
        }
        if let Some(s) = skip {
            query_parts.push(format!("$skip={}", s));
        }

        let query = query_parts.join("&");
        let url = self.build_url("EmpEmployment", Some(&query));

        info!("Fetching employment info from SuccessFactors");
        let response: ODataResponse<EmpEmployment> = self.get(&url).await?;
        Ok(response.data.results)
    }

    /// Get employee email information
    pub async fn get_per_email(&self, top: Option<u32>, skip: Option<u32>) -> Result<Vec<PerEmail>, ClientError> {
        let mut query_parts = vec![
            "$select=personIdExternal,emailType,emailAddress,isPrimary".to_string(),
        ];

        if let Some(t) = top {
            query_parts.push(format!("$top={}", t));
        }
        if let Some(s) = skip {
            query_parts.push(format!("$skip={}", s));
        }

        let query = query_parts.join("&");
        let url = self.build_url("PerEmail", Some(&query));

        info!("Fetching email info from SuccessFactors");
        let response: ODataResponse<PerEmail> = self.get(&url).await?;
        Ok(response.data.results)
    }

    /// Get employee phone information
    pub async fn get_per_phone(&self, top: Option<u32>, skip: Option<u32>) -> Result<Vec<PerPhone>, ClientError> {
        let mut query_parts = vec![
            "$select=personIdExternal,phoneType,phoneNumber,isPrimary".to_string(),
        ];

        if let Some(t) = top {
            query_parts.push(format!("$top={}", t));
        }
        if let Some(s) = skip {
            query_parts.push(format!("$skip={}", s));
        }

        let query = query_parts.join("&");
        let url = self.build_url("PerPhone", Some(&query));

        info!("Fetching phone info from SuccessFactors");
        let response: ODataResponse<PerPhone> = self.get(&url).await?;
        Ok(response.data.results)
    }

    /// Test the connection to SuccessFactors
    pub async fn test_connection(&self) -> Result<bool, ClientError> {
        let url = self.build_url("User", Some("$top=1"));

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ClientError::Request(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP client error: {0}")]
    HttpClient(String),
    #[error("Request error: {0}")]
    Request(String),
    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },
    #[error("Failed to parse response: {0}")]
    Parse(String),
}

impl ClientError {
    pub fn status_code(&self) -> u16 {
        match self {
            ClientError::Api { status, .. } => *status,
            _ => 500,
        }
    }
}
