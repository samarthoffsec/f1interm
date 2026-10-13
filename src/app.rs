mod event;
mod state;
mod network;
mod ui;

use clap::Parser;
use crossterm::event::{Event, KeyEventKind};
use futures::StreamExt;
use ratatui::backend::Backend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::{Duration, MissedTickBehavior, interval};
use tracing::{info, warn, instrument};
use color_eyre::Result;

use crate::event::{PanelFocus, UiState};
use crate::state::RaceState;
use crate::ui::{self, draw, render_footer};
use crate::network::{self, FeedSource, supervise, client::HttpClient};

pub struct App {
    state: RaceState,
    frozen: Option<RaceState>,
    ui: UiState,
    dirty: bool,
    quit: bool,
    cfg: Config,
}

impl App {
    pub fn new(cli: Cli, cfg: Config) -> Self {
        Self {
            state: RaceState::default(),
            frozen: None,
            ui: UiState::default(),
            dirty: true,
            quit: false,
            cfg,
        }
    }

    pub fn view(&self) -> &RaceState {
        self.frozen.as_ref().unwrap_or(&self.state)
    }

    fn toggle_pause(&mut self) {
        self.frozen = match self.frozen.take() {
            Some(_) => None,
            None => Some(self.state.clone()),
        };
    }

    fn next_driver(&mut self) {
        self.ui.selected_driver = (self.ui.selected_driver + 1) % 20; // capped by actual count
    }

    fn cycle_panel(&mut self) {
        self.ui.focus = match self.ui.focus {
            PanelFocus::Timing => PanelFocus::Telemetry,
            PanelFocus::Telemetry => PanelFocus::Status,
            PanelFocus::Status => PanelFocus::Compare,
            PanelFocus::Compare => PanelFocus::Timing,
        };
    }

    fn prev_driver(&mut self) {
        if self.ui.selected_driver > 0 {
            self.ui.selected_driver -= 1;
        } else {
            self.ui.selected_driver = 19; // wrap
        }
    }

    fn select_compare_a(&mut self) {
        self.ui.compare_driver_a = self.ui.selected_driver;
    }

    fn select_compare_b(&mut self) {
        self.ui.compare_driver_b = self.ui.selected_driver;
    }

    fn clear_compare(&mut self) {
        self.ui.compare_driver_a = 0;
        self.ui.compare_driver_b = 1;
    }

    async fn on_key(&mut self, key: Event, cmd_tx: &mpsc::Sender<Command>) -> Result<()> {
        use Event::Key;
        use KeyEventKind::Press;

        if key.kind != Press {
            return Ok(());
        }

        let key_code = key.code;

        match key_code {
            // Driver selection
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                self.prev_driver();
                self.dirty = true;
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                self.next_driver();
                self.dirty = true;
            }

            // Panel focus cycling
            crossterm::event::KeyCode::Tab => {
                self.cycle_panel();
                self.dirty = true;
            }
            crossterm::event::KeyCode::BackTab => {
                // Shift+Tab - cycle reverse
                self.cycle_panel();
                self.dirty = true;
            }

            // Panel number focus (1=Timing, 2=Telemetry, 3=Status, 4=Compare)
            '1' => {
                self.ui.focus = PanelFocus::Timing;
                self.dirty = true;
            }
            '2' => {
                self.ui.focus = PanelFocus::Telemetry;
                self.dirty = true;
            }
            '3' => {
                self.ui.focus = PanelFocus::Status;
                self.dirty = true;
            }
            '4' => {
                self.ui.focus = PanelFocus::Compare;
                self.dirty = true;
            }

            // Enter: set telemetry target to highlighted driver
            crossterm::event::KeyCode::Enter => {
                self.dirty = true;
            }

            // Comparison driver selection: c key
            crossterm::event::KeyCode::Char('c') => {
                // Toggle between driver A and B based on current selection
                // If comparing, press c again to set B
                if self.ui.compare_driver_a == 0 {
                    self.select_compare_a();
                } else if self.ui.compare_driver_b == 1 && self.ui.compare_driver_a != self.ui.selected_driver {
                    self.select_compare_b();
                }
                self.dirty = true;
            }
            crossterm::event::KeyCode::Char('C') | crossterm::event::KeyCode::Char('c') with_shift => {
                // Clear comparison (Shift+C)
                self.clear_compare();
                self.dirty = true;
            }

            // Pause/Resume: Space
            crossterm::event::KeyCode::Char(' ') | crossterm::event::KeyCode::Char('p') => {
                self.toggle_pause();
                self.dirty = true;
            }

            // Graph time window: +/-
            crossterm::event::KeyCode::Char('+') | crossterm::event::KeyCode::Char('=') => {
                self.ui.graph_window_secs = self.ui.graph_window_secs.saturating_add(15);
                self.dirty = true;
            }
            crossterm::event::KeyCode::Char('-') => {
                self.ui.graph_window_secs = self.ui.graph_window_secs.saturating_sub(15);
                self.dirty = true;
            }

            // Sort: s key
            crossterm::event::KeyCode::Char('s') => {
                // Cycle sort: position -> best lap -> last lap -> gap -> position
                self.dirty = true;
            }

            // Team colour / colour-blind toggle: t key
            crossterm::event::KeyCode::Char('t') => {
                // Toggle colour blind mode
                self.ui.colour_blind = !self.ui.colour_blind;
                self.dirty = true;
            }

            // ASCII mode toggle: z key
            crossterm::event::KeyCode::Char('z') => {
                self.cfg.ascii_mode = !self.cfg.ascii_mode;
                self.dirty = true;
            }

            // ASCII mode toggle: z key
            crossterm::event::KeyCode::Char('z') => {
                self.cfg.ascii_mode = !self.cfg.ascii_mode;
                self.dirty = true;
            }

            // Session picker: / key
            crossterm::event::KeyCode::Char('/') => {
                // Open session picker (placeholder - would show menu)
                self.dirty = true;
            }

            // Reconnect: r key
            crossterm::event::KeyCode::Char('r') => {
                // Force reconnect - send command
                let _ = cmd_tx.try_send(Command::Reconnect);
                self.dirty = true;
            }

            // Help: ? key
            crossterm::event::KeyCode::Char('?') => {
                // Help overlay (handled in draw)
                self.dirty = true;
            }

            // Quit: q or Ctrl+C
            'q' | crossterm::event::KeyCode::Char('c') if matches!(key.code, crossterm::event::KeyCode::Char('c')) => {
                self.quit = true;
            }

            _ => {}
        }

        Ok(())
    }

    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let (ev_tx, mut ev_rx) = mpsc::channel::<FeedEvent>(1024);
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);

        // Start the network source (mock by default)
        let cancel = tokio::sync::CancellationToken::new();
        let net_handle = tokio::spawn(network::start(
            self.cfg.source_type(),
            ev_tx,
            cmd_rx,
            cancel.clone(),
        ));

        let mut keys = crossterm::event::EventStream::new();
        let mut render_tick = interval(Duration::from_millis(33));
        render_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
        self.dirty = true;

        while !self.quit {
            // Process terminal input
            tokio::select! {
                Some(Ok(ev)) = keys.next() => {
                    self.on_key(ev, &cmd_tx).await;
                    self.dirty = true;
                }

                // Feed events: apply, then drain burst before next draw
                Some(first) = ev_rx.recv() => {
                    let st = RaceState::make_mut(&mut self.state);
                    st.apply(first);
                    while let Ok(more) = ev_rx.try_recv() {
                        st.apply(more);
                    }
                    if self.frozen.is_none() {
                        self.dirty = true;
                    }
                }
            }

            // Render tick
            tokio::select! {
                _ = render_tick.tick() => {
                    if self.dirty {
                        terminal.draw(|f| {
                            ui::draw(f, &self, self.cfg.no_color, self.cfg.ascii_mode)?;
                            render_footer(f, f.area(), &self.ui);
                        })?;
                        self.dirty = false;
                    }
                }
            }
        }

        // Clean shutdown
        cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(1), net_handle).await;
        
        Ok(())
    }
}

impl Config {
    fn source_type(&self) -> network::SourceType {
        // Determine source based on CLI flags and config
        // For now, return mock; Phase 2 will add live/OpenF1
        network::SourceType::Mock
    }
}