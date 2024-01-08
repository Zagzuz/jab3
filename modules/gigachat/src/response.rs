use crate::proto::GigaChatMessage;
use api::timestamp::{deserialize_ts_from_i64, deserialize_ts_from_millis, Timestamp};
use compact_str::CompactString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionsResponse {
    pub choices: Vec<GigaChatChoice>,
    #[serde(deserialize_with = "deserialize_ts_from_i64")]
    pub created: Timestamp,
    pub model: CompactString,
    pub usage: GigaChatUsage,
    pub object: CompactString,
}

#[derive(Debug, Deserialize)]
pub struct GigaChatChoice {
    pub message: GigaChatMessage,
    pub index: i32,
    pub finish_reason: CompactString,
}

#[derive(Debug, Deserialize)]
pub struct GigaChatUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: CompactString,
    #[serde(deserialize_with = "deserialize_ts_from_millis")]
    pub expires_at: Timestamp,
}
