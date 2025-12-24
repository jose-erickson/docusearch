use serde::{Deserialize, Serialize};

/// OData response wrapper for SuccessFactors API
#[derive(Debug, Deserialize)]
pub struct ODataResponse<T> {
    #[serde(rename = "d")]
    pub data: ODataResults<T>,
}

#[derive(Debug, Deserialize)]
pub struct ODataResults<T> {
    pub results: Vec<T>,
}

/// Single entity OData response
#[derive(Debug, Deserialize)]
pub struct ODataSingleResponse<T> {
    #[serde(rename = "d")]
    pub data: T,
}

/// Employee personal information from PerPersonal entity
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
    #[serde(rename = "nationality")]
    pub nationality: Option<String>,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: Option<String>,
    #[serde(rename = "maritalStatus")]
    pub marital_status: Option<String>,
}

/// Employee email information from PerEmail entity
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

/// Employee phone information from PerPhone entity
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

/// Employment information from EmpEmployment entity
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

/// Job information from EmpJob entity
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

/// User information from User entity
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

/// Combined employee data for API response
#[derive(Debug, Serialize, Clone)]
pub struct Employee {
    pub user_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub hire_date: Option<String>,
    pub status: Option<String>,
}

/// API error response
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

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
