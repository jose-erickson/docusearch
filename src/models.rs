use serde::{Deserialize, Serialize};

/// OData response wrapper for SuccessFactors list responses.
#[derive(Debug, Deserialize)]
pub struct ODataResponse<T> {
    #[serde(rename = "d")]
    pub data: ODataResults<T>,
}

#[derive(Debug, Deserialize)]
pub struct ODataResults<T> {
    pub results: Vec<T>,
}

/// OData response for single-entity lookups.
#[derive(Debug, Deserialize)]
pub struct ODataSingleResponse<T> {
    #[serde(rename = "d")]
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerPersonal {
    #[serde(rename = "personIdExternal")]
    pub person_id_external: Option<String>,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(rename = "middleName")]
    pub middle_name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub gender: Option<String>,
    pub nationality: Option<String>,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: Option<String>,
    #[serde(rename = "maritalStatus")]
    pub marital_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerEmail {
    #[serde(rename = "personIdExternal")]
    pub person_id_external: Option<String>,
    #[serde(rename = "emailType")]
    pub email_type: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerPhone {
    #[serde(rename = "personIdExternal")]
    pub person_id_external: Option<String>,
    #[serde(rename = "phoneType")]
    pub phone_type: Option<String>,
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmpEmployment {
    #[serde(rename = "personIdExternal")]
    pub person_id_external: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    #[serde(rename = "employmentStatus")]
    pub employment_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmpJob {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "jobCode")]
    pub job_code: Option<String>,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub division: Option<String>,
    pub location: Option<String>,
    #[serde(rename = "managerId")]
    pub manager_id: Option<String>,
    #[serde(rename = "costCenter")]
    pub cost_center: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub username: Option<String>,
    #[serde(rename = "defaultFullName")]
    pub default_full_name: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "hireDate")]
    pub hire_date: Option<String>,
}

/// Public error envelope returned to API consumers.
///
/// Only carries sanitized, non-sensitive information. Internal error details
/// (upstream URLs, stack traces, raw upstream bodies) are logged but never
/// included here.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}
