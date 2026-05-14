use serde::{Deserialize, Serialize};

/// One billable Claude Code API call extracted from a JSONL transcript line.
#[derive(Debug, Clone, Serialize)]
pub struct UsageRecord {
    pub message_id: String,
    pub session_id: String,
    pub project_path: String,
    pub timestamp: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_5m_tokens: u64,
    pub cache_1h_tokens: u64,
    pub cost_usd: f64,
    pub service_tier: Option<String>,
    pub speed: Option<String>,
}

/// Raw `message.usage` payload as emitted by Claude. Tolerant deserialization:
/// missing nested fields default to 0 so we never crash on a schema variation.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RawUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_creation: Option<CacheCreation>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub speed: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CacheCreation {
    #[serde(default)]
    pub ephemeral_5m_input_tokens: u64,
    #[serde(default)]
    pub ephemeral_1h_input_tokens: u64,
}

impl RawUsage {
    /// Returns true if no billable tokens are present (line is a no-op for cost).
    pub fn is_empty(&self) -> bool {
        self.input_tokens == 0
            && self.output_tokens == 0
            && self.cache_read_input_tokens == 0
            && self.cache_creation_input_tokens == 0
    }

    /// Returns (cache_5m, cache_1h) tokens. Falls back to lumping everything into 5m if the
    /// breakdown is not present (older or unusual schema variants).
    pub fn cache_split(&self) -> (u64, u64) {
        match &self.cache_creation {
            Some(c) => (c.ephemeral_5m_input_tokens, c.ephemeral_1h_input_tokens),
            None => (self.cache_creation_input_tokens, 0),
        }
    }
}

/// Top-level JSONL line. We only care about `type == "assistant"` lines; everything else
/// is skipped by the parser. Many fields are optional to remain robust to format drift.
#[derive(Debug, Clone, Deserialize)]
pub struct RawLine {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub message: Option<RawMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMessage {
    pub id: Option<String>,
    pub model: Option<String>,
    pub usage: Option<RawUsage>,
}
