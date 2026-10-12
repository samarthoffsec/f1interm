#[derive(Debug, Clone)]
pub enum FeedEvent {
    Session(SessionMeta),
    Drivers(Vec<DriverInfo>),
    Timing(Vec<TimingDelta>),
    Weather(WeatherSample),
    TrackStatus(TrackStatus),
    RaceControl(Vec<RaceControlMsg>),
    Clock(SessionClock),
    Link(LinkStatus),
}

#[derive(Debug, Clone)]
pub enum Command {
    Pause,
    Resume,
    SelectSession(String),
    Reconnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    Connecting { attempt: u32 },
    Live,
    Stale,
    Reconnecting { attempt: u32, retry_in: std::time::Duration },
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Race,
    Qualifying,
    Practice,
}

#[derive(Debug, Clone)]
pub struct SessionMeta {
    pub name: String,
    pub circuit: String,
    pub session_type: SessionType,
    pub total_laps: u16,
    pub current_lap: u16,
    pub time_remaining: Option<std::time::Duration>,
}

#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub number: u8,
    pub code: String,
    pub full_name: String,
    pub team: String,
    pub team_colour: Option<(u8, u8, u8)>,
}

#[derive(Debug, Clone)]
pub struct TimingDelta {
    pub position: u8,
    pub driver_code: String,
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
}

#[derive(Debug, Clone)]
pub struct WeatherSample {
    pub air_temp: f32,
    pub track_temp: f32,
    pub humidity: f32,
    pub wind_speed: f32,
    pub wind_direction: f32,
    pub rainfall: f32,
}

#[derive(Debug, Clone)]
pub struct TrackStatus {
    pub status: TrackFlag,
    pub sector: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackFlag {
    Green,
    Yellow,
    SafetyCar,
    VirtualSafetyCar,
    Red,
    Checkered,
}

#[derive(Debug, Clone)]
pub struct RaceControlMsg {
    pub time: std::time::Instant,
    pub car_number: u8,
    pub message: String,
    pub flag: Option<TrackFlag>,
}

#[derive(Debug, Clone)]
pub struct SessionKey {
    pub year: u16,
    pub meeting: String,
    pub session: SessionType,
}

#[derive(Debug, Clone)]
pub struct SessionClock {
    pub elapsed: std::time::Duration,
    pub remaining: std::time::Duration,
    pub running: bool,
}