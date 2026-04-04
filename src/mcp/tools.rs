use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckParams {
    /// Skip test step
    #[serde(default)]
    pub skip_tests: Option<bool>,
    /// Skip docs-gate step
    #[serde(default)]
    pub skip_docs_gate: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CanaryParams {
    /// Health check URL
    pub url: Option<String>,
    /// Timeout in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LearnAddParams {
    /// Learning message
    pub message: String,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LearnSearchParams {
    /// Search query
    pub query: String,
}
