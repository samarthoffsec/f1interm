use async_trait::async_trait;
use color_eyre::Result;
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

pub mod client;
pub mod openf1;
pub mod replay;

#[async_trait::async_trait]
pub trait FeedSource: Send + 'static {
    async fn run(
        &mut self,
        tx: mpsc::Sender<crate::event::FeedEvent>,
        cancel: std::sync::atomic::AtomicBool,
    ) -> Result<()>;
}