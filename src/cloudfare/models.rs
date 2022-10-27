use std::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VerifyResponse {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct ResultInfo {
    pub page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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