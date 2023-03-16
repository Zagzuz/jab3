use compact_str::{CompactString, ToCompactString};
use derive_more::Display;
use eyre::ensure;
use serde::{de::Error, Deserialize, Deserializer};
use serde_aux::field_attributes::deserialize_number_from_string;

use api::{basic_types::UserId, proto::ChatId};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Commands {
    #[serde(rename = "$value")]
    pub cmd_vec: Vec<Command>,
}

impl Commands {
    pub fn verify(&self) -> eyre::Result<()> {
        for cmd in &self.cmd_vec {
            ensure!(
                !cmd.desc.is_empty() && cmd.desc.len() <= 256,
                "description can only contain 1-256 characters"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Command {
    pub scope: BotCommandScope,
    pub name: CommandName,
    pub desc: CompactString,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BotCommandScope {
    Default(BotCommandScopeDefault),
    AllPrivateChats(BotCommandScopeAllPrivateChats),
    AllGroupChats(BotCommandScopeAllGroupChats),
    AllChatAdministrators(BotCommandScopeAllChatAdministrators),
    Chat(BotCommandScopeChat),
    ChatAdministrators(BotCommandScopeChatAdministrators),
    ChatMember(BotCommandScopeChatMember),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeDefault;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeAllPrivateChats;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeAllGroupChats;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeAllChatAdministrators;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeChat {
    pub chat_id: ChatId,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeChatAdministrators {
    pub chat_id: ChatId,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BotCommandScopeChatMember {
    pub chat_id: ChatId,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user_id: UserId,
}

#[derive(Debug, Deserialize, Display, Eq, Hash, PartialEq)]
#[serde(remote = "Self")]
pub enum CommandName {
    #[serde(
    alias = "please",
    alias = "плис",
    alias = "Плис",
    alias = "плиз",
    alias = "Плиз"
    )]
    Please,
    #[serde(
    alias = "pls",
    alias = "плс",
    alias = "Плс",
    alias = "плз",
    alias = "Плз"
    )]
    Pls,
    #[serde(alias = "set_day", alias = "днюха", alias = "др")]
    SetDay,
}

impl<'de> Deserialize<'de> for CommandName {
    fn deserialize<D>(deserializer: D) -> Result<CommandName, D::Error>
    where
        D: Deserializer<'de>,
    {
        let command_name = CommandName::deserialize(deserializer)?;
        let s = command_name.to_compact_string();
        if s.is_empty() || s.len() > 32 {
            return Err(Error::custom(
                "command name can only contain 1-32 characters",
            ));
        }
        Ok(command_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_command_name() {
        let name = serde_json::from_str::<CommandName>(r#""pls""#).unwrap();
        assert_eq!(name, CommandName::Pls);
        let name = serde_json::from_str::<CommandName>(r#""please""#).unwrap();
        assert_eq!(name, CommandName::Please);
        let name = serde_json::from_str::<CommandName>(r#""set_day""#).unwrap();
        assert_eq!(name, CommandName::SetDay);
    }
}
