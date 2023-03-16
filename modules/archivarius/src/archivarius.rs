use crate::{guess::ChatGuessInfo, message::ChatMessageInfo, user::UserInfo};
use api::{
    basic_types::ChatIntId,
    proto::{Message, ParseMode},
};
use async_trait::async_trait;
use bincode::{Decode, Encode};
use bot::{
    bot::command::BotCommandInfo,
    communicator::Communicate,
    module::{Module, PersistentModule},
    persistence::Persistence,
};
use compact_str::CompactString;
use eyre::{bail, eyre};
use itertools::Itertools;
use log::debug;
use rand::seq::IteratorRandom;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Default)]
pub struct Archivarius {
    chat_data: HashMap<ChatIntId, ChatData>,
}

#[derive(Debug, Default, Encode, Decode)]
struct ChatData {
    pub active_command: Option<ActiveCommand>,
    pub messages: HashSet<ChatMessageInfo>,
    pub guesses: ChatGuessInfo,
    pub users: HashSet<UserInfo>,
}

impl Archivarius {
    pub fn new() -> Self {
        Default::default()
    }

    fn save_message(&mut self, message: &Message) -> eyre::Result<()> {
        let Some(original_message) = &message.reply_to_message else {
            bail!("replied message does not exist, command message = {message:?}");
        };
        self.chat_data
            .entry(message.chat.id)
            .or_default()
            .messages
            .insert(original_message.as_ref().into());
        Ok(())
    }

    async fn forward(
        &mut self,
        comm: &dyn Communicate,
        dest_chat_id: ChatIntId,
    ) -> eyre::Result<Option<Message>> {
        let Some(address) = self.chat_data.get(&dest_chat_id).and_then(|data| {
            data.messages
                .iter()
                .choose(&mut rand::thread_rng())
                .map(|i| i.address())
        }) else {
            return Ok(None);
        };
        // fixme: remove non-existing messages
        Ok(Some(
            comm.forward_message(
                dest_chat_id.into(),
                address.chat_id.into(),
                address.message_id,
                None,
                None,
            )
                .await?
                .into_result()?,
        ))
    }

    async fn _tell_the_answer(
        chat_data: &ChatData,
        comm: &dyn Communicate,
        message: &Message,
    ) -> eyre::Result<()> {
        let Some(guess_message_info) = chat_data
            .guesses
            .message_id
            .and_then(|id| chat_data.messages.get(&ChatMessageInfo::new(id)))
            else {
                comm.reply_message(
                    "No messages to guess",
                    message.chat.id.into(),
                    message.message_id,
                    None,
                )
                    .await?
                    .into_result()?;
                return Ok(());
            };
        let Some(author_info) = guess_message_info.author_info.as_ref() else {
            bail!("no author info for guess message, info = {guess_message_info:?}");
        };
        let name = author_info
            .username
            .as_ref()
            .unwrap_or(&author_info.full_name);
        comm.reply_message(
            &(format!("That was `@{name}`") + r#"\!"#),
            message.chat.id.into(),
            message.message_id,
            Some(ParseMode::MarkdownV2),
        )
            .await?
            .into_result()?;
        Ok(())
    }

    async fn guess(&mut self, comm: &dyn Communicate, message: &Message) -> eyre::Result<()> {
        let Some(data) = self.chat_data.get_mut(&message.chat.id) else {
            comm.reply_message(
                "No messages to guess",
                message.chat.id.into(),
                message.message_id,
                None,
            )
                .await?
                .into_result()?;
            return Ok(());
        };

        // Self::tell_the_answer(&data, comm, message).await?;

        let Some(info) = data
            .messages
            .iter()
            .filter(|info| info.author_info.is_some())
            .choose(&mut rand::thread_rng())
            else {
                comm.reply_message(
                    "No messages to guess",
                    message.chat.id.into(),
                    message.message_id,
                    None,
                )
                    .await?
                    .into_result()?;
                return Ok(());
            };

        let address = info.address();

        comm.copy_message(
            message.chat.id.into(),
            None,
            address.chat_id.into(),
            address.message_id,
            None,
            None,
            vec![],
            None,
            None,
            None,
            None,
            None,
        )
            .await?
            .into_result()?;

        data.guesses.message_id.replace(address.message_id);

        Ok(())
    }

    async fn check_guess(
        &mut self,
        comm: &dyn Communicate,
        message: &Message,
    ) -> eyre::Result<bool> {
        let Some((users, messages, guess_info, guess_message_id)) =
            self.chat_data.get_mut(&message.chat.id).and_then(|d| {
                let message_id = d.guesses.message_id?;
                Some((&mut d.users, &d.messages, &mut d.guesses, message_id))
            })
            else {
                bail!("cannot check the guess: the game has not yet started");
            };

        let message_info = messages
            .get(&ChatMessageInfo::new(guess_message_id))
            .ok_or(eyre!(
                "guess message info not found, guess_message_id = {guess_message_id}"
            ))?;

        let author = message_info.author_info.as_ref().ok_or(eyre!(
            "cannot check the guess: the game has not yet started"
        ))?;

        debug!("{author:?} - {:?}", message.text);

        let text = message
            .text
            .as_ref()
            .ok_or(eyre!("not a text message, message = {message:?}"))?;

        if author == text.as_str() {
            let winner = message
                .from
                .as_ref()
                .ok_or(eyre!("cannot add points to a non-user entity"))?;
            guess_info.finish_game(winner.id);
            users.insert(author.clone());
            comm.reply_message(
                "Exactly! +1 point",
                message.chat.id.into(),
                message.message_id,
                None,
            )
                .await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn points(&self, message: &Message, comm: &dyn Communicate) -> eyre::Result<()> {
        let Some(data) = self.chat_data.get(&message.chat.id) else {
            comm.reply_message(
                "The list is empty",
                message.chat.id.into(),
                message.message_id,
                None,
            )
                .await?;
            return Ok(());
        };

        let leaders: CompactString = Itertools::intersperse(
            data.guesses
                .points
                .iter()
                .sorted_by(|(_, p1), (_, p2)| Ord::cmp(p2, p1))
                .filter_map(|(id, points)| {
                    let user_info = data.users.get(&UserInfo {
                        id: *id,
                        ..Default::default()
                    })?;
                    Some((
                        user_info.username.as_ref().unwrap_or(&user_info.full_name),
                        points,
                    ))
                })
                .map(|(name, score)| format!("{name} \t{score}")),
            "\n".into(),
        )
            .collect();
        comm.reply_message(
            &format!("```\n{leaders}\n```"),
            message.chat.id.into(),
            message.message_id,
            Some(ParseMode::MarkdownV2),
        )
            .await?;

        Ok(())
    }

    fn handle_active_command(&mut self, message: &Message) {
        let Some(active_command) = self
            .chat_data
            .get(&message.chat.id)
            .and_then(|d| d.active_command.as_ref())
            else {
                return;
            };
        match active_command {
            ActiveCommand::DevSave(chat_id) => {
                self.chat_data
                    .entry(*chat_id)
                    .or_default()
                    .messages
                    .insert(message.into());
            }
        };
    }
}

#[async_trait]
impl Module for Archivarius {
    async fn try_execute_command(
        &mut self,
        comm: &dyn Communicate,
        cmd: &BotCommandInfo,
        message: &Message,
    ) -> eyre::Result<()> {
        let command_name = match CommandName::from_str(cmd.name().as_str()) {
            Ok(name) => name,
            Err(err) => {
                debug!("{err}");
                self.handle_active_command(message);
                if let Err(err) = self.check_guess(comm, message).await {
                    debug!("{err}");
                }
                return Ok(());
            }
        };

        match command_name {
            CommandName::Forward => {
                if self.forward(comm, message.chat.id).await?.is_none() {
                    comm.reply_message(
                        "No messages saved",
                        message.chat.id.into(),
                        message.message_id,
                        None,
                    )
                        .await?
                        .into_result()?;
                }
            }
            CommandName::Save => {
                self.save_message(message)?;
                comm.reply_message("Saved!", message.chat.id.into(), message.message_id, None)
                    .await?;
            }
            CommandName::Guess => {
                self.guess(comm, message).await?;
            }
            CommandName::Remove => {
                if let Some(messages) = self
                    .chat_data
                    .get_mut(&message.chat.id)
                    .map(|d| &mut d.messages)
                {
                    messages.remove(&message.into());
                }
                comm.reply_message("Done!", message.chat.id.into(), message.message_id, None)
                    .await?;
            }
            CommandName::Points => {
                self.points(message, comm).await?;
            }
            CommandName::DevSave => {
                let Ok(chat_id) = cmd.query().parse::<ChatIntId>() else {
                    comm.reply_message(
                        &format!(
                            "failed to parse '{}' as chat id to save new messages",
                            cmd.query()
                        ),
                        message.chat.id.into(),
                        message.message_id,
                        None,
                    )
                        .await?
                        .into_result()?;
                    return Ok(());
                };
                comm.reply_message(
                    &format!("listening to messages for '{chat_id}' now..."),
                    message.chat.id.into(),
                    message.message_id,
                    None,
                )
                    .await?
                    .into_result()?;
                self.chat_data
                    .entry(message.chat.id)
                    .or_default()
                    .active_command
                    .replace(ActiveCommand::DevSave(chat_id));
            }
            CommandName::DevStop => {
                if self
                    .chat_data
                    .get(&message.chat.id)
                    .and_then(|d| d.active_command.as_ref())
                    .is_none()
                {
                    comm.reply_message(
                        "no active command",
                        message.chat.id.into(),
                        message.message_id,
                        None,
                    )
                        .await?
                        .into_result()?;
                } else {
                    self.chat_data
                        .entry(message.chat.id)
                        .or_default()
                        .active_command = None;
                    comm.reply_message("done", message.chat.id.into(), message.message_id, None)
                        .await?
                        .into_result()?;
                }
            }
        }
        Ok(())
    }
}

impl Persistence for Archivarius {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    fn serialize(&self) -> eyre::Result<Self::Output> {
        Ok(bincode::encode_to_vec(
            &self.chat_data,
            bincode::config::standard(),
        )?)
    }

    fn deserialize(&mut self, bytes: Self::Input) -> eyre::Result<()>
        where
            Self: Sized,
    {
        self.chat_data = bincode::decode_from_slice::<HashMap<ChatIntId, ChatData>, _>(
            bytes.as_slice(),
            bincode::config::standard(),
        )?
            .0;
        Ok(())
    }
}

impl PersistentModule for Archivarius {}

enum CommandName {
    Save,
    Forward,
    Guess,
    Remove,
    Points,
    DevSave,
    DevStop,
}

impl FromStr for CommandName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "forward" => Ok(CommandName::Forward),
            "guess" => Ok(CommandName::Guess),
            "save" => Ok(CommandName::Save),
            "remove" => Ok(CommandName::Remove),
            "points" => Ok(CommandName::Points),
            "dev_save" => Ok(CommandName::DevSave),
            "dev_stop" => Ok(CommandName::DevStop),
            _ => {
                bail!("failed to recognize '{s}' as a possible command")
            }
        }
    }
}

#[derive(Debug, Encode, Decode)]
enum ActiveCommand {
    DevSave(ChatIntId),
}
