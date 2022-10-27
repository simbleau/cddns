use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VerifyResponse {
    pub success: bool,
}
