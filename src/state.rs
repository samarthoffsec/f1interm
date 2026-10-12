use crate::event::{DriverInfo, LinkStatus, RaceControlMsg, SessionClock, SessionMeta, SessionType, TimingDelta, TrackFlag, TrackStatus, WeatherSample};
use std::collections::BTreeMap;

pub type DriverNo = u8;

#[derive(Debug, Clone, Default)]
pub struct DriverState {
    pub car: DriverInfo,
    pub gap_to_leader: Option<f32>,
    pub interval_to_ahead: Option<f32>,
    pub last_lap: f32,
    pub best_lap: f32,
    pub s1: f32,
    pub s2: f32,
    pub s3: f32,
    pub in_pit: bool,
    pub tyres: String,
    pub drs: bool,
    pub position: u8,
}

#[derive(Debug, Clone, Default)]
pub struct RaceState {
    pub session: Option<SessionMeta>,
    pub drivers: BTreeMap<DriverNo, DriverState>,
    pub order: Vec<DriverNo>,
    pub weather: Option<WeatherSample>,
    pub track: TrackStatus,
    pub clock: SessionClock,
    pub link: LinkStatus,
    pub race_control: BTreeMap<std::time::Instant, RaceControlMsg>,
}

impl RaceState {
    pub fn apply(&mut self, ev: crate::event::FeedEvent) {
        use crate::event::FeedEvent::*;
        match ev {
            Session(meta) => {
                self.session = Some(meta);
                self.order = (1..=meta.total_laps).map(|_| 0).collect();
            }
            Drivers(drivers) => {
                for d in drivers {
                    let state = self.drivers.entry(d.number).or_default();
                    state.car = d.clone();
                    if d.position > 0 {
                        state.position = d.position;
                    }
                }
                self.reorder();
            }
            Timing(deltas) => {
                for t in deltas {
                    if let Some(state) = self.drivers.get_mut(&t.driver_code.parse::<u8>().unwrap_or(0)) {
                        state.gap_to_leader = t.gap_to_leader;
                        state.interval_to_ahead = t.interval_to_ahead;
                        state.last_lap = t.last_lap;
                        state.best_lap = t.best_lap;
                        state.s1 = t.s1;
                        state.s2 = t.s2;
                        state.s3 = t.s3;
                        state.in_pit = t.in_pit;
                        state.drs = t.drs;
                        state.tyres = t.tyres.clone();
                    }
                }
            }
            Weather(sample) => {
                self.weather = Some(sample);
            }
            TrackStatus(status) => {
                self.track = status;
            }
            Clock(clock) => {
                self.clock = clock;
            }
            Link(link) => {
                self.link = link;
            }
            RaceControl(messages) => {
                for msg in messages {
                    self.race_control.insert(msg.time, msg);
                    if self.race_control.len() > 100 {
                        let oldest = self.race_control.keys().min().unwrap();
                        self.race_control.remove(oldest);
                    }
                }
            }
        }
    }

    fn reorder(&mut self) {
        let mut sorted: Vec<(DriverNo, &DriverState)> = self.drivers.iter().collect();
        sorted.sort_by(|a, b| b.1.position.cmp(&a.1.position));
        self.order = sorted.iter().map(|(no, _)| *no).collect();
    }
}