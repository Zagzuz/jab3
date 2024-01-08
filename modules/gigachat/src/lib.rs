use crate::{
    endpoints::ChatCompletions,
    proto::GigaChatMessage,
    request::ChatCompletionsRequest,
    response::{AccessTokenResponse, ChatCompletionsResponse},
};
use api::{
    endpoints::Endpoint,
    params::{
        eyre,
        eyre::{bail, ensure, eyre},
    },
    proto::{ChatAction, Message, ParseMode},
    timestamp::Timestamp,
};
use async_trait::async_trait;
use bot::{
    bot::command::BotCommandInfo,
    communicator::Communicate,
    module::{Module, PersistentModule},
    persistence::Persistence,
};
use compact_str::CompactString;
use derive_more::Display;
use log::debug;
use reqwest::{Certificate, Client};
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

mod endpoints;
mod proto;
mod request;
mod response;

pub struct GigaChat {
    https_url: CompactString,
    token_request_url: CompactString,
    token_expires_at: Timestamp,
    access_token: CompactString,
    uuid: Uuid,
    cert: Certificate,
    messages: Vec<GigaChatMessage>,
}

impl GigaChat {
    pub fn new() -> Self {
        let work_dir = std::env::var("WORK_DIR").expect("work dir not found");
        let path =
            std::path::Path::new(&work_dir).join("modules/gigachat/russian_trusted_root_ca.cer");
        let buf = std::fs::read(&path)
            .unwrap_or_else(|err| panic!("cert not found on path '{path:?}', {err}"));
        let cert = Certificate::from_pem(&buf).expect("wrong certificate format");
        Self {
            https_url: "https://gigachat.devices.sberbank.ru/api/v1".into(),
            token_request_url: "https://ngw.devices.sberbank.ru:9443/api/v2/oauth".into(),
            token_expires_at: Timestamp::now(),
            access_token: Default::default(),
            uuid: Uuid::new_v4(),
            cert,
            messages: vec![],
        }
    }
    pub async fn update_token_if_expired(&mut self) -> eyre::Result<()> {
        if self.token_expires_at > Timestamp::now() + Timestamp::from(5) {
            return Ok(());
        }

        let client_id =
            std::env::var("GIGACHAT_CLIENT_ID").map_err(|err| eyre!("client id {err:?}"))?;
        let client_secret = std::env::var("GIGACHAT_CLIENT_SECRET")
            .map_err(|err| eyre!("client secret {err:?}"))?;

        #[derive(Serialize)]
        struct Data {
            pub scope: CompactString,
        }

        let client = Client::builder()
            .add_root_certificate(self.cert.clone())
            .build()?;
        let response: AccessTokenResponse = client
            .post(self.token_request_url.as_str())
            .basic_auth(client_id, Some(client_secret))
            .header("RqUID", &self.uuid.to_string())
            .form(&Data {
                scope: "GIGACHAT_API_PERS".into(),
            })
            .send()
            .await?
            .json()
            .await?;
        self.token_expires_at = response.expires_at;
        self.access_token = response.access_token;
        Ok(())
    }

    pub async fn chat_completions(&mut self, query: &str) -> eyre::Result<ChatCompletionsResponse> {
        self.update_token_if_expired().await?;

        let url = format!("{}/{}", self.https_url, ChatCompletions::PATH);

        let history = self
            .messages
            .iter()
            .cloned()
            .rev()
            .take(100)
            .collect::<Vec<_>>();
        let data = ChatCompletionsRequest::with_history_latest(history, query);

        let client = Client::builder()
            .add_root_certificate(self.cert.clone())
            .build()?;
        let text = client
            .request(ChatCompletions::METHOD, url)
            .bearer_auth(&self.access_token)
            .json(&data)
            .send()
            .await?
            .text()
            .await?;
        let response: ChatCompletionsResponse =
            serde_json::from_str(&text).map_err(|err| eyre!("'{text}', {err:?}"))?;
        Ok(response)
    }
}

#[derive(Debug, Display, Copy, Clone)]
enum CommandName {
    Ask,
    CarCrash,
}

impl FromStr for CommandName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gpt" => Ok(CommandName::Ask),
            "гпт" => Ok(CommandName::Ask),
            "жпт" => Ok(CommandName::Ask),
            "car_crash" => Ok(CommandName::CarCrash),
            _ => {
                bail!("failed to recognize '{s}' as a possible command")
            }
        }
    }
}

#[async_trait]
impl Module for GigaChat {
    async fn try_execute_command(
        &mut self,
        comm: &dyn Communicate,
        cmd: &BotCommandInfo,
        message: &Message,
    ) -> eyre::Result<()> {
        match CommandName::from_str(cmd.name().as_str()) {
            Ok(CommandName::Ask) => {
                let response = self.chat_completions(cmd.query()).await?;

                ensure!(!response.choices.is_empty(), "no answer for {cmd:?}");
                comm.send_chat_action(message.chat.id.into(), None, ChatAction::Typing)
                    .await?;

                let answer = response
                    .choices
                    .iter()
                    .map(|choice| choice.message.content.clone())
                    .collect::<Vec<_>>()
                    .join(" ");
                debug!("gigachat answer: {answer}");

                debug!(
                    "gigachat finish reason: {:?}",
                    response.choices.last().unwrap().finish_reason
                );

                let parse_mode: Option<ParseMode> = if answer.contains("<img src") {
                    comm.reply_message(
                        "Unfortunately, I cannot post an image here.",
                        message.chat.id.into(),
                        message.message_id,
                        None,
                    )
                    .await?;
                    return Ok(());
                } else if answer.contains("```") {
                    Some(ParseMode::MarkdownV2)
                } else {
                    None
                };

                comm.reply_message(
                    &answer,
                    message.chat.id.into(),
                    message.message_id,
                    parse_mode,
                )
                .await?;

                let mut messages = response
                    .choices
                    .into_iter()
                    .map(|choice| choice.message)
                    .collect::<Vec<_>>();
                self.messages.append(&mut messages);
            }
            Ok(CommandName::CarCrash) => {
                self.messages.clear();
                comm.reply_message(
                    "Ouch! What happened? Can't remember anything.",
                    message.chat.id.into(),
                    message.message_id,
                    None,
                )
                .await?;
            }
            Err(err) => {
                debug!("{err}");
            }
        }
        Ok(())
    }
}

impl Persistence for GigaChat {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    fn serialize(&self) -> eyre::Result<Self::Output> {
        Ok(bincode::encode_to_vec(
            (self.token_expires_at.millis(), self.access_token.as_str()),
            bincode::config::standard(),
        )?)
    }

    fn deserialize(&mut self, input: Self::Input) -> eyre::Result<()> {
        let (expires_at, token) = bincode::decode_from_slice::<(i128, String), _>(
            input.as_slice(),
            bincode::config::standard(),
        )?
        .0;
        self.access_token = token.into();
        self.token_expires_at = Timestamp::from_millis(expires_at);
        Ok(())
    }
}

impl PersistentModule for GigaChat {}

#[cfg(test)]
mod test {
    use crate::GigaChat;
    use api::timestamp::Timestamp;
    use dotenv::dotenv;

    #[tokio::test]
    async fn get_new_token() {
        dotenv().ok();
        let mut gigachat = GigaChat::new();
        gigachat.update_token_if_expired().await.unwrap();
        assert!(gigachat.token_expires_at <= Timestamp::now() + Timestamp::from(1860));
        assert!(gigachat.token_expires_at > Timestamp::now() + Timestamp::from(1740))
    }
}
