use api::proto::{Message, MessageEntity, MessageEntityType};
use compact_str::CompactString;
use eyre::bail;

#[derive(Debug)]
pub struct BotCommandInfo {
    name: CompactString,
    query: CompactString,
}

impl TryFrom<&Message> for BotCommandInfo {
    type Error = eyre::Report;

    fn try_from(message: &Message) -> Result<Self, Self::Error> {
        let Some(text) = message.text.as_ref() else {
            bail!("no text for bot command in {message:?}");
        };
        if let Some(entity) = message.is_of_entity(MessageEntityType::BotCommand) {
            Ok(Self::from_command(text, entity))
        } else {
            Ok(Self::from_text(text))
        }
    }
}

impl BotCommandInfo {
    pub fn name(&self) -> &CompactString {
        &self.name
    }

    pub fn query(&self) -> &CompactString {
        &self.query
    }

    fn from_command(text: &CompactString, bot_command_entity: MessageEntity) -> Self {
        let (cmd, query) = text.split_at(bot_command_entity.length);
        let cmd = cmd
            .strip_prefix('/')
            .and_then(|c| c.split('@').next())
            .unwrap_or(cmd);
        Self {
            name: cmd.into(),
            query: query.into(),
        }
    }

    fn from_text(text: &CompactString) -> Self {
        let (cmd, query) = text.split_once(' ').unwrap_or((text.as_str(), ""));
        let cmd = cmd.split('@').next().unwrap_or(cmd);
        Self {
            name: cmd.into(),
            query: query.into(),
        }
    }
}
