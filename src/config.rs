use std::path::PathBuf;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Config {
    pub openf1_token: String,
    pub default_session: Option<String>,
    pub graph_window_secs: u64,
    pub units: String,
    pub timezone: String,
    pub log_level: String,
    pub no_color: bool,
    pub ascii_mode: bool,
}

impl Config {
    pub fn load(cli: &crate::cli::Cli) -> color_eyre::Result<Self> {
        let mut cfg = Self::from_env()?;

        // CLI flags override env
        if cli.openf1_token != "" {
            cfg.openf1_token = cli.openf1_token.clone();
        }
        if cli.no_color {
            cfg.no_color = true;
        }
        if cli.ascii_mode {
            cfg.ascii_mode = true;
        }

        // Config file path: ~/.config/f1interm/config.toml (fallback to current dir)
        let config_file = std::env::current_dir()
            .map(|d| d.join("f1interm_config.toml"))
            .unwrap_or(PathBuf::from("f1interm_config.toml"));

        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            let file_cfg: toml::Value = content.parse()?;
            Self::apply_toml(&mut cfg, &file_cfg);
        }

        Ok(cfg)
    }

    fn from_env() -> color_eyre::Result<Self> {
        Ok(Self {
            openf1_token: std::env::var("OPENF1_TOKEN").unwrap_or_default(),
            default_session: None,
            graph_window_secs: 60,
            units: "metric".to_string(),
            timezone: "UTC".to_string(),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            no_color: std::env::var("NO_COLOR").is_ok(),
            ascii_mode: std::env::var("F1INTERM_ASCII").is_ok(),
        })
    }

    fn apply_toml(cfg: &mut Self, val: &toml::Value) {
        if let Some(v) = val.get("openf1_token") {
            cfg.openf1_token = v.as_str().unwrap_or("").to_string();
        }
        if let Some(v) = val.get("default_session") {
            cfg.default_session = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = val.get("graph_window_secs") {
            cfg.graph_window_secs = v.as_integer().unwrap_or(60) as u64;
        }
        if let Some(v) = val.get("units") {
            cfg.units = v.as_str().unwrap_or("metric").to_string();
        }
        if let Some(v) = val.get("timezone") {
            cfg.timezone = v.as_str().unwrap_or("UTC").to_string();
        }
        if let Some(v) = val.get("log_level") {
            cfg.log_level = v.as_str().unwrap_or("info").to_string();
        }
        if let Some(v) = val.get("no_color") {
            cfg.no_color = v.as_bool().unwrap_or(false);
        }
        if let Some(v) = val.get("ascii_mode") {
            cfg.ascii_mode = v.as_bool().unwrap_or(false);
        }
    }
}