use crate::connector::ConnectorMode;
use api::proto::UpdateType;
use compact_str::CompactString;
use std::{collections::HashSet, path::PathBuf};

#[derive(Debug)]
pub struct BotConfig {
    pub allowed_updates: HashSet<UpdateType>,
    pub update_limit: Option<u32>,
    pub polling_timeout: Option<u32>,
    pub skip_missed_updates: bool,
    pub work_dir: PathBuf,
    pub data_file_name: CompactString,
    pub connector_mode: ConnectorMode,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            allowed_updates: Default::default(),
            update_limit: None,
            polling_timeout: None,
            skip_missed_updates: false,
            work_dir: Default::default(),
            data_file_name: "jab.data".into(),
            connector_mode: Default::default(),
        }
    }
}
