use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct CloudflareError {
    pub code: i32,
    pub message: String,
    pub error_chain: Option<Vec<CloudflareMessage>>,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareMessage {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResponse {
    pub success: bool,
    pub errors: Vec<CloudflareError>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyResponse {
    pub success: bool,
    pub messages: Vec<CloudflareMessage>,
}

#[derive(Debug, Deserialize)]
pub struct ResultInfo {
    pub page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub status: String,
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.id)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub locked: bool,
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} => {}", self.name, self.id, self.content)
    }
}

#[derive(Debug, Deserialize)]
pub struct ListZonesResponse {
    pub success: bool,
    pub result_info: ResultInfo,
    pub result: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
pub struct ListRecordsResponse {
    pub success: bool,
    pub result_info: ResultInfo,
    pub result: Vec<Record>,
}

#[derive(Debug, Deserialize)]
pub struct PatchRecordResponse {
    pub success: bool,
    pub result: Record,
}
