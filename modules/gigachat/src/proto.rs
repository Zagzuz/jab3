use compact_str::CompactString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GigaChatRole {
    Assistant,
    User,
    System,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GigaChatMessage {
    pub role: GigaChatRole,
    pub content: CompactString,
}
