use async_trait::async_trait;
use color_eyre::Result;
use futures::StreamExt;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::info;
use crate::event::{self, FeedEvent};
use crate::state::RaceState;

#[derive(Debug, Deserialize)]
pub struct ReplayFrame {
    pub t_ms: u64,
    pub event: String,
}

pub struct ReplaySource {
    pub frames: Vec<ReplayFrame>,
    pub speed: f64,
    pub start_offset: Duration,
}

impl ReplaySource {
    pub fn from_ndjson(path: &str, speed: f64) -> color_eyre::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let frames: Vec<ReplayFrame> = serde_json::from_str(&content)?;
        Ok(Self {
            frames,
            speed: speed.max(0.1),
            start_offset: Duration::from_millis(0),
        })
    }
}

#[async_trait::async_trait]
impl FeedSource for ReplaySource {
    async fn run(&mut self, tx: mpsc::Sender<FeedEvent>, cancel: &std::sync::atomic::AtomicBool) -> Result<()> {
        let mut last_tick = Instant::now();
        let mut frame_idx = 0usize;

        info!("Replay started: {} frames at {:.1}x speed", self.frames.len(), self.speed);

        while !cancel.load(std::sync::atomic::Ordering::Relaxed) && frame_idx < self.frames.len() {
            let now = Instant::now();
            let elapsed = now.duration_since(last_tick);
            let target_delay = std::time::Duration::from_millis(
                (1000f64 / (30.0 * self.speed)) as u64,
            );

            if elapsed >= target_delay {
                let frame = &self.frames[frame_idx];
                Self::process_replay_frame(frame, &tx).await;
                frame_idx += 1;
                last_tick = now;
            } else {
                tokio::time::sleep(target_delay - elapsed).await;
                let frame = &self.frames[frame_idx];
                Self::process_replay_frame(frame, &tx).await;
                frame_idx += 1;
                last_tick = Instant::now();
            }
        }

        info!("Replay finished: {} frames processed", frame_idx);
        Ok(())
    }

    async fn process_replay_frame(frame: &ReplayFrame, tx: &mpsc::Sender<FeedEvent>) {
        let event_name = frame.event.as_str();

        match event_name {
            "Position" => {
                let _ = tx.try_send(FeedEvent::Timing(vec![event::TimingDelta {
                    position: 1,
                    driver_code: String::new(),
                    gap_to_leader: None,
                    interval_to_ahead: None,
                    last_lap: 0.0,
                    best_lap: 0.0,
                    s1: 0.0,
                    s2: 0.0,
                    s3: 0.0,
                    in_pit: false,
                    tyres: String::new(),
                    drs: false,
                }]));
            }
            "Weather" => {
                let _ = tx.try_send(FeedEvent::Weather(event::WeatherSample {
                    air_temp: 24.0,
                    track_temp: 38.0,
                    humidity: 52.0,
                    wind_speed: 2.0,
                    wind_direction: 180.0,
                    rainfall: 0.0,
                }));
            }
            "Session" => {
                let _ = tx.try_send(FeedEvent::Session(event::SessionMeta {
                    name: "Race".to_string(),
                    circuit: String::new(),
                    session_type: event::SessionType::Race,
                    total_laps: 53,
                    current_lap: 1,
                    time_remaining: None,
                }));
            }
            _ => {}
        }
    }
}