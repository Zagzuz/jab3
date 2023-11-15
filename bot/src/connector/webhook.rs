use crate::connector::Connector;
use api::{
    endpoints::SetWebhook,
    proto::{CommonUpdate, UpdateType},
    request::SetWebhookRequest,
};
use async_trait::async_trait;
use compact_str::CompactString;
use eyre::eyre;
use log::debug;
use tokio::{io::AsyncReadExt, net::TcpListener};

pub struct WebhookConnector {
    token: CompactString,
    listener: Option<TcpListener>,
    buffer: Vec<u8>,
    config: WebhookConnectorConfig,
}

#[derive(Default)]
pub struct WebhookConnectorConfig {
    pub https_url: Option<CompactString>,
    pub ip_address: Option<CompactString>,
    pub drop_pending_updates: bool,
    pub max_connections: Option<i32>,
    pub allowed_updates: Vec<UpdateType>,
}

impl WebhookConnector {
    pub(crate) fn with_config(token: &str, config: WebhookConnectorConfig) -> Self {
        Self {
            token: token.into(),
            listener: None,
            buffer: vec![],
            config,
        }
    }
}

#[async_trait]
impl Connector for WebhookConnector {
    async fn on_startup(&mut self) -> eyre::Result<()> {
        let addr = match self.config.ip_address.as_ref() {
            None => "127.0.0.1:443".into(),
            Some(ip) => format!("{ip}:443"),
        };
        self.listener.replace(TcpListener::bind(&addr).await?);
        debug!("jab is listening on {addr}...");
        let request = SetWebhookRequest {
            ip_address: self.config.ip_address.clone(),
            url: self.config.https_url.clone().unwrap_or_default(),
            max_connections: self.config.max_connections,
            allowed_updates: Some(self.config.allowed_updates.clone()),
            drop_pending_updates: Some(self.config.drop_pending_updates),
            ..Default::default()
        };
        <WebhookConnector as Connector>::send_request::<SetWebhook>(
            self.token.as_str(),
            &request,
            None,
        )
        .await?
        .into_result()?;
        Ok(())
    }

    async fn fetch_updates(&mut self) -> eyre::Result<Vec<CommonUpdate>> {
        let listener = self
            .listener
            .as_mut()
            .ok_or(eyre!("webhook connector is not listening"))?;
        let (mut stream, _addr) = listener.accept().await?;
        stream.read_buf(&mut self.buffer).await?;
        let update = serde_json::from_slice::<CommonUpdate>(self.buffer.as_slice())?;
        self.buffer.clear();
        Ok(vec![update])
    }
}
