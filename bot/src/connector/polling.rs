use crate::connector::Connector;
use api::{
    endpoints::GetUpdates,
    proto::{CommonUpdate, UpdateType},
    request::GetUpdatesRequest,
};
use async_trait::async_trait;
use compact_str::{CompactString, ToCompactString};

pub struct PollingConnector {
    token: CompactString,
    last_update_id: Option<usize>,
    config: PollingConnectorConfig,
}

#[derive(Default)]
pub struct PollingConnectorConfig {
    pub allowed_updates: Vec<UpdateType>,
    pub limit: Option<u32>,
    pub timeout: Option<u32>,
}

impl PollingConnector {
    pub fn with_config(token: &str, config: PollingConnectorConfig) -> Self {
        Self {
            token: token.to_compact_string(),
            last_update_id: None,
            config,
        }
    }
}

#[async_trait]
impl Connector for PollingConnector {
    async fn on_startup(&mut self) -> eyre::Result<()> {
        Ok(())
    }

    async fn fetch_updates(&mut self) -> eyre::Result<Vec<CommonUpdate>> {
        let request = GetUpdatesRequest {
            offset: self.last_update_id,
            limit: self.config.limit,
            timeout: self.config.timeout,
            allowed_updates: Some(self.config.allowed_updates.clone()),
        };

        let updates = <PollingConnector as Connector>::send_request::<GetUpdates>(
            self.token.as_str(),
            &request,
            None,
        )
        .await?
        .into_result()?;

        if !updates.is_empty() {
            let last_update_id = updates.iter().map(|u| u.id).max().unwrap();
            self.last_update_id.replace(last_update_id as usize);
        };

        Ok(updates)
    }
}
