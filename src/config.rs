use bot::connector::ConnectorMode;
use compact_str::CompactString;
use eyre::ensure;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub connector_mode: ConnectorMode,
    pub data_file_name: CompactString,
    #[serde(default)]
    pub skip_missed_updates: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            connector_mode: Default::default(),
            data_file_name: "jab3.data".into(),
            skip_missed_updates: false,
        }
    }
}

impl GlobalConfig {
    fn validate(&self) -> eyre::Result<()> {
        ensure!(
            !self.data_file_name.is_empty(),
            "data file name cannot be empty"
        );
        Ok(())
    }

    pub fn from_file(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = serde_xml_rs::from_str::<Self>(contents.as_str())?;
        config.validate()?;
        Ok(config)
    }
}

/*
impl ConnectorConfig {
    pub fn verify(&self) -> eyre::Result<()> {
        ensure!(
            matches!(self.update_limit, Some(1..=100)),
            "number of updates to retrieve is strictly 1-100"
        );
        if matches!(self.timeout, Some(0)) {
            warn!(
                "timeout = 0, i.e. short polling; should be positive,
                short polling should be used for testing purposes only"
            );
        }
        Ok(())
    }
}*/

/*#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct AllowedUpdates {
    #[serde(rename = "$value")]
    pub upd_vec: Vec<UpdateTypeItem>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct UpdateTypeItem {
    #[serde(rename = "type")]
    pub update_type: UpdateType,
}

#[derive(Default, Debug, Deserialize, PartialEq, Eq)]
pub struct BirthminderConfig {
    pub timezone: Timezone,
}*/

/*#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        command::{
            BotCommandScope, BotCommandScopeChatMember, BotCommandScopeDefault, Command,
            CommandName,
        },
        connector::config::ConnectorConfig,
    };
    use api::proto::ChatId;
    use imager::config::ImagerConfig;
    use std::process::Command;

    #[test]
    fn load_template_config() {
        let connector = ConnectorConfig {
            allowed_updates: Some(AllowedUpdates {
                upd_vec: vec![
                    UpdateTypeItem {
                        update_type: UpdateType::Message,
                    },
                    UpdateTypeItem {
                        update_type: UpdateType::EditedMessage,
                    },
                    UpdateTypeItem {
                        update_type: UpdateType::ChannelPost,
                    },
                ],
            }),
            limit: Some(100),
            rps: 30,
            timeout: None,
            update_channel_size: 1024,
        };
        let commands = Commands {
            cmd_vec: vec![
                Command {
                    scope: BotCommandScope::Default(BotCommandScopeDefault),
                    name: CommandName::SetDay,
                    desc: "dd.mm.yyyy format".into(),
                },
                Command {
                    scope: BotCommandScope::Default(BotCommandScopeDefault),
                    name: CommandName::Please,
                    desc: "ordered google image search".into(),
                },
                Command {
                    scope: BotCommandScope::ChatMember(BotCommandScopeChatMember {
                        chat_id: ChatId::from(-1001367947767),
                        user_id: 5363186157,
                    }),
                    name: CommandName::Pls,
                    desc: "randomised google image search".into(),
                },
            ],
        };
        let imager = ImagerConfig { limit: 100 };
        let birthminder = BirthminderConfig {
            timezone: Timezone::Utc,
        };
        let config = GlobalConfig {
            connector,
            commands,
            imager,
            birthminder,
        };
        assert_eq!(
            config,
            GlobalConfig::from_file("bot/test/config.template.xml").unwrap()
        )
    }
}
*/
