# f1interm - Implementation Plan (Phase 1)

## Project Initialized
- Rust project `f1interm` v0.1.0, edition 2021
- Dependencies: color-eyre, tracing, clap, tokio, tokio-util, futures, async-trait, ratatui 0.25, crossterm 0.27, serde, serde_json, chrono, rand 0.8
- Project structure established in `/Users/ravishankarkullaiah/f1interm/f1interm/`

## Phase 1 Status: COMPLETE

**Goal:** a runnable skeleton that proves the architecture.

### Completed:
1. **Crate setup** - Cargo.toml with all dependencies, `color-eyre`, `tracing`, `clap`
2. **main.rs** - Entrypoint with `color-eyre::install()` panic hook, terminal init/restore, `Cli::parse()`
3. **event.rs** - Core type definitions: `FeedEvent`, `Command`, `LinkStatus`, `SessionType`, `SessionMeta`, `DriverInfo`, `TimingDelta`, `WeatherSample`, `TrackStatus`, `TrackFlag`, `RaceControlMsg`, `SessionKey`, `SessionClock`, `FeedSource` trait
4. **state.rs** - `RaceState` struct with pure `apply()` reducer (idempotent, order-tolerant merge)
5. **network.rs** - `MockSource` implementing `FeedSource`, emitting synthetic 20-car timing + telemetry at realistic cadence
6. **app.rs** - `App` struct with `run()` method, `select!` loop over render tick + input + feed channel, key handling
7. **ui.rs** - Basic TUI draw function with layout tiers and placeholder widgets
8. **cli.rs** - CLI args: `--live`, `--mock`, `--quit`
9. **Test criteria met** - Binary compiles and runs; `f1interm --help` works; `f1interm --mock` runs; `f1interm --quit` exits cleanly

### Exit criteria verified:
- Mock race runs correctly
- Resize never panics (basic test passed)
- `q` always restores the terminal

### Next phases (from PRD):
- Phase 2: Live OpenF1/WebSocket integration
- Phase 3: Advanced widgets (BrailleGraph, sparklines, team colours)
- Phase 4: Polish, profiling & packaging