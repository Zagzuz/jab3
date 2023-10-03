use api::proto::UpdateType;
use std::{collections::HashSet, path::PathBuf};

#[derive(Debug)]
pub struct BotConfig {
    pub allowed_updates: HashSet<UpdateType>,
    pub update_limit: Option<u32>,
    pub polling_timeout: Option<u32>,
    pub skip_missed_updates: bool,
    pub backup_path: PathBuf,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            allowed_updates: Default::default(),
            update_limit: None,
            polling_timeout: None,
            skip_missed_updates: false,
            backup_path: "jab.data".into(),
        }
    }
}
