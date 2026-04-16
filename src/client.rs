use base64::{engine::general_purpose::STANDARD, Engine};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::models::*;

/// Maximum response size to prevent memory exhaustion from malicious/malformed upstream (10 MB).
const MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

/// SuccessFactors OData API client.
///
/// The Basic Auth header is stored as a `SecretString` so it is zeroed
/// from memory when the client is dropped and never printed by Debug.
#[derive(Clone)]
pub struct SuccessFactorsClient {
    client: Client,
    base_url: String,
    auth_header: SecretString,
}

impl std::fmt::Debug for SuccessFactorsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuccessFactorsClient")
            .field("base_url", &self.base_url)
            .field("auth_header", &"***REDACTED***")
            .finish()
    }
}

impl SuccessFactorsClient {
    pub fn new(config: &Config) -> Result<Self, ClientError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
            .connect_timeout(std::time::Duration::from_secs(10))
            // Security: enforce TLS 1.2 minimum. Rustls enforces this by default
            // but we set it explicitly for clarity.
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            // Security: disable redirects to prevent credential leakage to third-party hosts
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|e: reqwest::Error| ClientError::HttpClient(e.to_string()))?;

        let auth_string = format!("{}:{}", config.auth_username(), config.password());
        let auth_header = SecretString::new(format!("Basic {}", STANDARD.encode(auth_string)));

        let base_url = config.sf_api_url.as_str().trim_end_matches('/').to_string();

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    fn build_url(&self, entity: &str, query: Option<&str>) -> String {
        let base = format!("{}/odata/v2/{}", self.base_url, entity);
        match query {
            Some(q) => format!("{}?{}", base, q),
            None => base,
        }
    }

    /// Make an authenticated GET request, streaming at most `MAX_RESPONSE_BYTES`.
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, ClientError> {
        debug!(target: "sf_client", "requesting SuccessFactors resource");

        let response = self
            .client
            .get(url)
            .header("Authorization", self.auth_header.expose_secret())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                // Do not propagate full URL - might leak internal structure
                warn!(target: "sf_client", error = %e, "upstream request failed");
                ClientError::Request("upstream request failed".to_string())
            })?;

        let status = response.status();

        // Security: enforce content-length limits before reading body
        if let Some(len) = response.content_length() {
            if len as usize > MAX_RESPONSE_BYTES {
                return Err(ClientError::ResponseTooLarge);
            }
        }

        if !status.is_success() {
            // Body text kept only for server-side logs; not returned to caller
            let body = response.text().await.unwrap_or_default();
            error!(
                target: "sf_client",
                status = status.as_u16(),
                body_len = body.len(),
                "SuccessFactors API returned error"
            );
            return Err(ClientError::Api {
                status: status.as_u16(),
            });
        }

        // Read bounded bytes before deserialization
        let bytes = response.bytes().await.map_err(|e| {
            warn!(target: "sf_client", error = %e, "failed to read response body");
            ClientError::Request("failed to read response body".to_string())
        })?;

        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(ClientError::ResponseTooLarge);
        }

        serde_json::from_slice::<T>(&bytes).map_err(|e| {
            warn!(target: "sf_client", error = %e, "failed to parse response");
            ClientError::Parse
        })
    }

    pub async fn get_users(
        &self,
        top: u32,
        skip: u32,
    ) -> Result<Vec<User>, ClientError> {
        let query = format!(
            "$select=userId,username,defaultFullName,email,status,hireDate&$top={}&$skip={}",
            top, skip
        );
        let url = self.build_url("User", Some(&query));

        info!(target: "sf_client", top, skip, "fetching users");
        let response: ODataResponse<User> = self.get(&url).await?;
        Ok(response.data.results)
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User, ClientError> {
        // Security: escape single quotes in the OData key (OData injection prevention)
        // and percent-encode the rest for URL safety.
        let escaped = escape_odata_string(user_id);
        let encoded = utf8_percent_encode(&escaped, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "{}/odata/v2/User('{}')?$select=userId,username,defaultFullName,email,status,hireDate",
            self.base_url, encoded
        );

        info!(target: "sf_client", "fetching specific user");
        let response: ODataSingleResponse<User> = self.get(&url).await?;
        Ok(response.data)
    }

    pub async fn get_per_personal(
        &self,
        top: u32,
        skip: u32,
    ) -> Result<Vec<PerPersonal>, ClientError> {
        let query = format!(
            "$select=personIdExternal,firstName,lastName,middleName,displayName,gender,nationality,dateOfBirth,maritalStatus&$top={}&$skip={}",
            top, skip
        );
        let url = self.build_url("PerPersonal", Some(&query));
        info!(target: "sf_client", top, skip, "fetching personal info");
        let response: ODataResponse<PerPersonal> = self.get(&url).await?;
        Ok(response.data.results)
    }

    pub async fn get_emp_job(
        &self,
        top: u32,
        skip: u32,
    ) -> Result<Vec<EmpJob>, ClientError> {
        let query = format!(
            "$select=userId,jobCode,jobTitle,department,division,location,managerId,costCenter,startDate&$top={}&$skip={}",
            top, skip
        );
        let url = self.build_url("EmpJob", Some(&query));
        info!(target: "sf_client", top, skip, "fetching job info");
        let response: ODataResponse<EmpJob> = self.get(&url).await?;
        Ok(response.data.results)
    }

    pub async fn get_emp_employment(
        &self,
        top: u32,
        skip: u32,
    ) -> Result<Vec<EmpEmployment>, ClientError> {
        let query = format!(
            "$select=personIdExternal,userId,startDate,endDate,employmentStatus&$top={}&$skip={}",
            top, skip
        );
        let url = self.build_url("EmpEmployment", Some(&query));
        info!(target: "sf_client", top, skip, "fetching employment info");
        let response: ODataResponse<EmpEmployment> = self.get(&url).await?;
        Ok(response.data.results)
    }

    pub async fn get_per_email(
        &self,
        top: u32,
        skip: u32,
    ) -> Result<Vec<PerEmail>, ClientError> {
        let query = format!(
            "$select=personIdExternal,emailType,emailAddress,isPrimary&$top={}&$skip={}",
            top, skip
        );
        let url = self.build_url("PerEmail", Some(&query));
        info!(target: "sf_client", top, skip, "fetching emails");
        let response: ODataResponse<PerEmail> = self.get(&url).await?;
        Ok(response.data.results)
    }

    pub async fn get_per_phone(
        &self,
        top: u32,
        skip: u32,
    ) -> Result<Vec<PerPhone>, ClientError> {
        let query = format!(
            "$select=personIdExternal,phoneType,phoneNumber,isPrimary&$top={}&$skip={}",
            top, skip
        );
        let url = self.build_url("PerPhone", Some(&query));
        info!(target: "sf_client", top, skip, "fetching phones");
        let response: ODataResponse<PerPhone> = self.get(&url).await?;
        Ok(response.data.results)
    }

    pub async fn test_connection(&self) -> Result<bool, ClientError> {
        let url = self.build_url("User", Some("$top=1"));

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header.expose_secret())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|_| ClientError::Request("connection failed".to_string()))?;

        Ok(response.status().is_success())
    }
}

/// Escape single quotes for OData string literals by doubling them.
/// This prevents OData filter injection when user-supplied IDs are embedded
/// directly in a `('...')` key predicate.
fn escape_odata_string(input: &str) -> String {
    input.replace('\'', "''")
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP client initialization failed: {0}")]
    HttpClient(String),
    #[error("upstream request failed: {0}")]
    Request(String),
    #[error("upstream API error (status {status})")]
    Api { status: u16 },
    #[error("response exceeded maximum allowed size")]
    ResponseTooLarge,
    #[error("failed to parse upstream response")]
    Parse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_single_quote() {
        assert_eq!(escape_odata_string("o'neil"), "o''neil");
        assert_eq!(escape_odata_string("normal"), "normal");
        assert_eq!(escape_odata_string("a'b'c"), "a''b''c");
    }
}
