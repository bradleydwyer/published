use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    Available,
    Taken,
    Unknown,
}

impl std::fmt::Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Availability::Available => write!(f, "AVAILABLE"),
            Availability::Taken => write!(f, "TAKEN"),
            Availability::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreResult {
    pub store_id: String,
    pub store_name: String,
    pub available: Availability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browse_url: Option<String>,
    pub elapsed_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub summary: Summary,
    pub results: Vec<StoreResult>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub available: usize,
    pub taken: usize,
    pub unknown: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreInfo {
    pub id: String,
    pub name: String,
    pub platform: String,
}
