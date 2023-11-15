pub(crate) mod config;
pub(crate) mod polling;
pub(crate) mod webhook;

use async_trait::async_trait;

use eyre::eyre;
use http::HeaderMap;

use serde::{Deserialize, Serialize};

use api::{
    endpoints::Endpoint,
    files::GetFiles,
    params::ToParams,
    proto::{CommonUpdate, InputFileResult},
    response::CommonResponse,
};

const BASE_URL: &str = "https://api.telegram.org";

#[async_trait]
pub trait Connector {
    async fn on_startup(&mut self) -> eyre::Result<()>;

    async fn fetch_updates(&mut self) -> eyre::Result<Vec<CommonUpdate>>;

    fn query_url<E: Endpoint>(token: &str) -> String
    where
        Self: Sized,
    {
        format!("{}/bot{}/{}", BASE_URL, token, E::PATH)
    }

    async fn send_request<E>(
        token: &str,
        data: &E::Request,
        headers: Option<HeaderMap>,
    ) -> eyre::Result<CommonResponse<E::Response>>
    where
        Self: Sized,
        E: Endpoint,
        E::Request: Serialize + Sync,
        E::Response: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let url = Self::query_url::<E>(token);
        let client = reqwest::Client::new();
        let request = client
            .request(E::METHOD, url)
            .headers(headers.unwrap_or_default())
            .json(data)
            .build()?;
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

    async fn send_multipart<E>(
        token: &str,
        data: &E::Request,
        headers: Option<HeaderMap>,
    ) -> eyre::Result<CommonResponse<E::Response>>
    where
        E: Endpoint,
        E::Request: Serialize + GetFiles,
        E::Response: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let url = Self::query_url::<E>(token);

        let mut form = reqwest::multipart::Form::new();
        for (field_name, field_value) in data.to_params()? {
            form = form.part(
                field_name,
                reqwest::multipart::Part::text(field_value.to_string()),
            );
        }
        for (file_name, file) in data.get_files() {
            form = match file.data().await? {
                InputFileResult::Text(text) => {
                    form.part(file_name, reqwest::multipart::Part::text(text))
                }
                InputFileResult::Part(part) => form.part(file_name, part),
            };
        }

        let client = reqwest::Client::new();
        let request = client
            .request(E::METHOD, url)
            .headers(headers.unwrap_or_default())
            .multipart(form)
            .build()?;
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
}
