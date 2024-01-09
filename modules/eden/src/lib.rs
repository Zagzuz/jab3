mod endpoints;
mod request;
mod response;

use crate::{
    endpoints::ImageGeneration,
    request::{
        ImageGenerationProvider, ImageGenerationRequest, ImageGenerationSettings, OpenAIModels,
        Resolution,
    },
    response::{EdenResponse, ImageGenerationResponse, ImageGenerationResult, Status},
};
use api::{
    basic_types::ChatIntId,
    endpoints::Endpoint,
    params::{
        eyre,
        eyre::{bail, eyre},
    },
    proto::{ChatAction, Message},
};
use async_trait::async_trait;
use bot::{
    bot::command::BotCommandInfo,
    communicator::Communicate,
    module::{Module, PersistentModule},
    persistence::Persistence,
};
use compact_str::{CompactString, ToCompactString};
use log::{debug, info};
use reqwest::Client;
use std::{collections::HashMap, str::FromStr};

pub struct Eden {
    https_url: CompactString,
    last_query: HashMap<ChatIntId, CompactString>,
}

impl Eden {
    pub fn new() -> Self {
        Self {
            https_url: "https://api.edenai.run".into(),
            last_query: Default::default(),
        }
    }

    pub async fn gen_images_url(
        &mut self,
        query: &str,
        num: u8,
    ) -> eyre::Result<Vec<CompactString>> {
        let url = format!("{}/{}", self.https_url, &ImageGeneration::PATH);
        let settings = ImageGenerationSettings(
            [(ImageGenerationProvider::OpenAI, OpenAIModels::Dalle3)].into(),
        );
        let data = ImageGenerationRequest::new(
            vec![ImageGenerationProvider::OpenAI].into(),
            Some(
                vec![
                    ImageGenerationProvider::DeepAI,
                    ImageGenerationProvider::StabilityAI,
                    ImageGenerationProvider::Replicate,
                ]
                .into(),
            ),
            false,
            settings,
            query.into(),
            Resolution::Res1024_1024,
            num,
        );
        let token =
            std::env::var("EDEN_AI_API_KEY").map_err(|err| eyre!("eden ai api key {err}"))?;
        let text = Client::new()
            .post(url)
            .bearer_auth(token)
            .json(&data)
            .send()
            .await?
            .text()
            .await?;
        let response =
            serde_json::from_str::<EdenResponse>(&text).map_err(|err| eyre!("{text}, {err}"))?;
        let results = match response {
            EdenResponse::ImageGenerationResponse(r) => r.0,
            EdenResponse::Error(err) => {
                bail!(
                    "{}, {:?}",
                    err.error.r#type,
                    err.error.message.fallback_providers
                );
            }
        };
        let mut vs = Vec::new();
        for result in results.into_values() {
            match result {
                ImageGenerationResult::Fail(fail) => {
                    debug!("{}", fail.error.message);
                    continue;
                }
                ImageGenerationResult::Success(info) => {
                    vs.extend(info.items.into_iter().map(|item| item.image_resource_url));
                }
            }
        }
        Ok(vs)
    }
}

enum CommandName {
    Draw,
}

impl FromStr for CommandName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draw" => Ok(CommandName::Draw),
            _ => bail!("failed to recognize '{s}' as a possible command"),
        }
    }
}

#[async_trait]
impl Module for Eden {
    async fn try_execute_command(
        &mut self,
        comm: &dyn Communicate,
        cmd: &BotCommandInfo,
        message: &Message,
    ) -> eyre::Result<()> {
        match CommandName::from_str(cmd.name()) {
            Ok(CommandName::Draw) => {
                let query = if cmd.query().is_empty() {
                    match self.last_query.get(&message.chat.id) {
                        None => {
                            debug!("no query to draw");
                            return Ok(());
                        }
                        Some(last) => last,
                    }
                } else {
                    self.last_query.insert(message.chat.id, cmd.query().clone());
                    cmd.query()
                }
                .clone();
                comm.send_chat_action(message.chat.id.into(), None, ChatAction::UploadPhoto)
                    .await?
                    .into_result()?;
                let num = 1;
                let urls = self.gen_images_url(&query, num).await?;
                if urls.is_empty() {
                    comm.reply_message(
                        "Sorry, I cannot generate an image for the query specified",
                        message.chat.id.into(),
                        message.message_id,
                        None,
                    )
                    .await?
                    .into_result()?;
                    return Ok(());
                }
                for url in urls.iter().take(num as usize) {
                    comm.send_photo_url(
                        url.as_str(),
                        message.chat.id.into(),
                        Some(message.message_id),
                    )
                    .await?
                    .into_result()?;
                    debug!("{query} image url: {url}");
                }
            }
            Err(err) => {
                debug!("{err}");
            }
        }
        Ok(())
    }
}

impl Persistence for Eden {
    type Input = Vec<u8>;
    type Output = Vec<u8>;

    fn serialize(&self) -> eyre::Result<Self::Output> {
        Ok(bincode::encode_to_vec(
            self.last_query
                .iter()
                .map(|(chat_id, query)| (chat_id, query.as_str()))
                .collect::<HashMap<_, _>>(),
            bincode::config::standard(),
        )?)
    }

    fn deserialize(&mut self, input: Self::Input) -> eyre::Result<()> {
        let last_query = bincode::decode_from_slice::<HashMap<ChatIntId, String>, _>(
            input.as_slice(),
            bincode::config::standard(),
        )?
        .0;
        self.last_query = last_query
            .into_iter()
            .map(|(chat_id, query)| (chat_id, query.to_compact_string()))
            .collect();
        Ok(())
    }
}

impl PersistentModule for Eden {}
