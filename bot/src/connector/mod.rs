pub(crate) mod config;

use std::collections::HashSet;

use compact_str::{CompactString, ToCompactString};
use eyre::eyre;

use serde::{Deserialize, Serialize};

use api::{
    endpoints::{Endpoint, GetUpdates},
    proto::{CommonUpdate, UpdateType},
    request::GetUpdatesRequest,
    response::CommonResponse,
};

const BASE_URL: &str = "https://api.telegram.org";

pub struct Connector {
    token: CompactString,
    last_update_id: Option<usize>,
    update_request_config: UpdateRequestConfig,
}

#[derive(Default, Debug)]
pub struct UpdateRequestConfig {
    pub allowed_updates: HashSet<UpdateType>,
    pub limit: Option<u32>,
    pub timeout: Option<u32>,
}

impl UpdateRequestConfig {
    pub fn make_request(&self, offset: Option<usize>) -> GetUpdatesRequest {
        GetUpdatesRequest {
            offset,
            limit: self.limit,
            timeout: self.timeout,
            allowed_updates: Some(self.allowed_updates.clone().into_iter().collect()),
        }
    }
}

impl Connector {
    pub fn with_config(token: &str, update_request_config: UpdateRequestConfig) -> Self {
        Self {
            token: token.to_compact_string(),
            last_update_id: None,
            update_request_config,
        }
    }

    fn query_url<E: Endpoint>(token: &str) -> String {
        format!("{}/bot{}/{}", BASE_URL, token, E::PATH)
    }

    pub(crate) async fn send_request<E>(
        token: &str,
        data: &E::Request,
    ) -> eyre::Result<CommonResponse<E::Response>>
    where
        E: Endpoint,
        E::Request: Serialize,
        E::Response: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let url = Self::query_url::<E>(token);
        let client = reqwest::Client::new();
        let request = client.request(E::METHOD, url).json(data).build()?;
        let text = client.execute(request).await?.text().await?;
        let response =
            serde_json::from_str::<CommonResponse<E::Response>>(&text).map_err(|err| {
                eyre!(
                    "{}, type = {:?}, response = {}",
                    err,
                    std::any::type_name::<CommonResponse<E::Response>>(),
                    text
                )
            })?;
        Ok(response)
    }

    pub async fn recv(&mut self) -> eyre::Result<Vec<CommonUpdate>> {
        let request = self.update_request_config.make_request(self.last_update_id);

        let updates = Self::send_request::<GetUpdates>(&self.token, &request)
            .await?
            .into_result()?;

        if !updates.is_empty() {
            let last_update_id = updates.iter().map(|u| u.id).max().unwrap();
            self.last_update_id.replace(last_update_id as usize);
        };

        Ok(updates)
    }
}
