use api::proto::UpdateType;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ConnectorConfig {
    pub allowed_updates: HashSet<UpdateType>,
    pub update_limit: Option<u32>,
    pub timeout: Option<u32>,
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            allowed_updates: [UpdateType::Message].into(),
            update_limit: Some(10),
            timeout: Some(10),
        }
    }
}
