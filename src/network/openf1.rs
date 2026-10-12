use async_trait::async_trait;
use color_eyre::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::connect;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{info, warn};
use crate::event::{self, FeedEvent, DriverInfo, LinkStatus, RaceControlMsg, SessionClock, SessionMeta, SessionType, TimingDelta, TrackFlag, TrackStatus, WeatherSample};
use crate::model::dtos::*;
use crate::network::client::HttpClient;
use crate::state::RaceState;

pub struct SessionKey {
    pub year: u16,
    pub meeting: String,
    pub session: SessionType,
}

pub struct OpenF1Source {
    pub http: HttpClient,
    pub session_key: Option<SessionKey>,
    pub restart_after_bootstrap: bool,
}

impl OpenF1Source {
    pub fn new(http: HttpClient) -> Self {
        Self {
            http,
            session_key: None,
            restart_after_bootstrap: true,
        }
    }

    #[async_trait::async_trait]
    impl FeedSource for OpenF1Source {
        async fn run(&mut self, tx: mpsc::Sender<FeedEvent>, cancel: std::sync::atomic::AtomicBool) -> Result<()> {
            info!("OpenF1Source running");
            Ok(())
        }
    }
}