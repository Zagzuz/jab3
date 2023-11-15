use crate::connector::Connector;
use api::{
    endpoints::{Empty, GetWebhookInfo, SetWebhook},
    proto::{CommonUpdate, InputFile, UpdateType},
    request::SetWebhookRequest,
};
use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use compact_str::{CompactString, ToCompactString};
use eyre::{bail, ensure, eyre};
use http::StatusCode;
use log::{debug, trace};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub struct WebhookConnector {
    config: WebhookConnectorConfig,
    token: CompactString,
    rx: Option<UnboundedReceiver<eyre::Result<CommonUpdate>>>,
}

#[derive(Default)]
pub struct WebhookConnectorConfig {
    pub https_url: CompactString,
    pub ip_address: Option<CompactString>,
    pub drop_pending_updates: bool,
    pub max_connections: Option<i32>,
    pub allowed_updates: Vec<UpdateType>,
}

impl WebhookConnector {
    pub(crate) fn with_config(token: &str, config: WebhookConnectorConfig) -> Self {
        Self {
            token: token.into(),
            config,
            rx: None,
        }
    }
}

#[async_trait]
impl Connector for WebhookConnector {
    async fn on_startup(&mut self) -> eyre::Result<()> {
        let addr = SocketAddr::new(
            match self.config.ip_address.as_ref() {
                None => Ipv4Addr::new(127, 0, 0, 1),
                Some(ip) => Ipv4Addr::from_str(&ip)?,
            }
            .into(),
            443,
        );

        let work_dir = std::env::var("WORK_DIR").expect("WORK_DIR not set");
        let cert_path = PathBuf::from(&work_dir)
            .join("self_signed_certs")
            .join("cert.pem");
        let certificate = Some(InputFile::FilePath(
            cert_path
                .to_str()
                .ok_or(eyre!("failed to get cert path"))?
                .to_compact_string(),
        ));

        let (tx, rx) = unbounded_channel();

        let app = Router::new()
            .route(
                "/",
                post(move |Json(payload): Json<CommonUpdate>| async move {
                    debug!("webhook update received: {:?}", payload);
                    tx.send(Ok(payload)).expect("failed to send webhook update");
                    StatusCode::OK
                }),
            )
            .route(
                "/health-check",
                get(|| async {
                    trace!("health check request received");
                    StatusCode::OK
                }),
            );

        self.rx.replace(rx);

        let config = RustlsConfig::from_pem_file(
            cert_path,
            PathBuf::from(work_dir)
                .join("self_signed_certs")
                .join("key.pem"),
        )
        .await?;

        let srv = axum_server::bind_rustls(addr, config).serve(app.into_make_service());

        tokio::spawn(srv);

        debug!("jab is listening on {addr:?}...");

        let request = SetWebhookRequest {
            url: self.config.https_url.clone(),
            ip_address: self.config.ip_address.clone(),
            certificate,
            max_connections: self.config.max_connections,
            allowed_updates: Some(self.config.allowed_updates.clone()),
            drop_pending_updates: Some(self.config.drop_pending_updates),
            ..Default::default()
        };
        let webhook_is_set = <WebhookConnector as Connector>::send_multipart::<SetWebhook>(
            self.token.as_str(),
            &request,
            None,
        )
        .await?
        .into_result()?;

        ensure!(webhook_is_set, "webhook not set");

        let info = <WebhookConnector as Connector>::send_request::<GetWebhookInfo>(
            &self.token,
            &Empty,
            None,
        )
        .await?
        .into_result()?;
        debug!("webhook info: {info:?}");

        ensure!(
            info.has_custom_certificate,
            "webhook set without certificate"
        );
        ensure!(info.url == self.config.https_url, "wrong webhook https url");
        ensure!(
            info.ip_address == self.config.ip_address,
            "wrong webhook ip address"
        );

        Ok(())
    }

    async fn fetch_updates(&mut self) -> eyre::Result<Vec<CommonUpdate>> {
        let Some(rx) = self.rx.as_mut() else {
            bail!("uninitialized connector")
        };
        let update = rx.recv().await.expect("update channel died")?;
        Ok(vec![update])
    }
}
