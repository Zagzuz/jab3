use crate::proto::{GigaChatMessage, GigaChatRole};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Default, Serialize)]
#[skip_serializing_none]
pub struct ChatCompletionsRequest {
    pub model: GigaChatModel,
    pub messages: Vec<GigaChatMessage>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub n: Option<i64>,
    pub stream: bool,
    pub max_tokens: Option<i64>,
    pub repetition_penalty: Option<f32>,
    pub update_interval: Option<f32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum GigaChatModel {
    #[serde(rename = "GigaChat:latest")]
    #[default]
    Latest,
}

impl ChatCompletionsRequest {
    pub fn latest(mut messages: Vec<GigaChatMessage>, new_message: &str) -> Self {
        messages.push(GigaChatMessage {
            role: GigaChatRole::User,
            content: new_message.into(),
        });
        Self {
            model: GigaChatModel::Latest,
            messages,
            ..Default::default()
        }
    }
}
