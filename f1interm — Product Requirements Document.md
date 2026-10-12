# f1interm (`f1-btop-tui`) — Product Requirements Document

*A real-time Formula 1 terminal dashboard in Rust, in the spirit of btop. Version 0.1 (draft) · Author role: Principal Systems Architect / Rust / TUI design*

---

## 1. Executive Summary & Vision

### 1.1 Overview

`f1interm` is a single-binary, keyboard-driven terminal application that shows live (and replayed) Formula 1 session data in a dense, btop-style dashboard: a timing tower, live telemetry graphs, session/track/weather status, and a two-driver comparison view. It is written in Rust on `ratatui` + `crossterm` + `tokio`, and is designed to run comfortably in a tmux pane next to your editor during a race weekend.

### 1.2 Target Audience

- **F1 enthusiasts** who want timing-screen depth without a browser tab or a second monitor.
- **Developers and data nerds** who want a hackable, well-structured reference for async Rust + TUI + live feeds.
- **Terminal power users** (Alacritty, Kitty, Foot, Ghostty, iTerm2, Windows Terminal) who value keyboard-first, low-overhead tools.

### 1.3 Design Philosophy

1. **Low latency:** data-to-cell under 100 ms from receipt of a frame, excluding upstream feed delay.
2. **Zero bloat:** one static binary, no runtime, no browser engine, no database. Small, audited dependency tree.
3. **High visual fidelity:** truecolor team palettes, Braille/block-character graphs, box-drawing panels, consistent with btop's rounded-border, dense-information look.
4. **Never freeze:** network I/O, parsing, and rendering are strictly decoupled. A slow or dead socket must never block a keypress or a redraw.
5. **Honest data:** every panel can show a staleness indicator; missing data is shown as `--`, never as a guess.

### 1.4 Non-Goals (v1)

- Betting, fantasy, or prediction features.
- Video, audio, or team-radio playback (metadata only, if at all).
- A GUI, web, or mobile front end.
- Redistribution of F1 data; the app is a client only (see §8 Risks).

### 1.5 Success Metrics

| Metric | Target |
| --- | --- |
| Steady-state CPU (live race, 20 cars) | < 2% of one core |
| Resident memory | < 30 MB |
| Render rate | 30 FPS target, 60 FPS capable, event-driven redraw |
| Input-to-redraw latency | < 50 ms (p99) |
| Reconnect after socket drop | < 5 s median, no manual action |
| Cold start to first frame | < 300 ms (before network) |

---

## 2. System Architecture & Data Flow

### 2.1 Principles

- **Single-writer state.** Exactly one task (the UI/app task) owns and mutates `AppState`. Network tasks never touch it; they only send immutable messages.
- **Message passing over shared locks.** Bounded `tokio::sync::mpsc` channels connect producers to the app task. No `Arc<Mutex<AppState>>` on the hot path.
- **Copy-on-write snapshots.** State is held as `Arc<RaceState>`. Updates use `Arc::make_mut`; a pause or comparison view simply holds another `Arc` clone, which is O(1) until the next write.
- **Render only when dirty, capped at the frame budget.** A fixed-interval tick checks a `dirty` flag, so a burst of 200 messages causes one redraw, not 200.

### 2.2 Component Diagram

```text
┌──────────────────────────── tokio multi-thread runtime ────────────────────────────┐
│                                                                                │
│  ┌──────────────┐  raw frames   ┌───────────────┐  FeedEvent   ┌──────────────────┐ │
│  │ Feed Source  │ ─────────────▶ │ Decoder/Parser│ ────────────▶ │                  │ │
│  │ (WS / HTTP / │               │ (serde_json,  │  mpsc(1024)  │    App task      │ │
│  │  OpenF1)     │               │  typed DTOs)  │              │  AppState (Arc)  │ │
│  └──────────────┘               └───────────────┘              │  + UiState       │ │
│        ▲                                                       │                  │ │
│        │ respawn / backoff                                     │   select! {      │ │
│  ┌─────┴────────┐   Command (pause/seek/session)  ◀─────────┤     input,       │ │
│  │ Supervisor   │                                               │     feed rx,     │ │
│  └──────────────┘                                               │     render tick  │ │
│                                                                │   }              │ │
│  ┌──────────────┐  Key / Resize                                  │        │         │ │
│  │ crossterm    │ ─────────────────────────────────────────────▶│        ▼         │ │
│  │ EventStream  │                                               │ terminal.draw()  │ │
│  └──────────────┘                                               │ (ratatui, 33 ms) │ │
│                                                                 └─────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Task Topology

| Task | Responsibility | Blocking allowed? |
| --- | --- | --- |
| `app` (main) | `select!` over input, feed channel, render tick; owns state; draws | No |
| `feed_supervisor` | Spawns/cancels the active source; exponential-backoff reconnect | No |
| `feed_source` | WebSocket/HTTP I/O; emits raw frames | No (async I/O only) |
| `decoder` (inline in source, or `spawn_blocking` for heavy JSON bursts) | Decode, validate, normalize to `FeedEvent` | CPU-bound work off the reactor if > 1 ms |
| `replay` (optional) | Reads a recorded session file, emits events at original cadence × speed | No |

### 2.4 Data Sources

**Source: OpenF1** (`https://api.openf1.org/v1/`). JSON endpoints include `sessions`, `meetings`, `drivers`, `position`, `intervals`, `laps`, `stints`, `pit`, `car_data`, `weather`, `race_control`, and `location`. Historical REST is the basis for replay mode and for bootstrapping initial state. Live push (WebSocket/MQTT) may require an authenticated account; **verify current access terms and rate limits before Phase 2** and treat auth as a configurable token (`OPENF1_TOKEN`).

**Offline: Recorded sessions.** A simple NDJSON capture format (`{"t_ms":123456,"event":{...}}`) enables deterministic replay, offline development, and regression tests.

### 2.5 Normalized Event Model

All sources are adapted into one internal vocabulary; the UI never sees source-specific JSON.

```text
FeedEvent
 ├─ Session(SessionMeta)              // name, circuit, type, start, total laps
 ├─ Drivers(Vec<DriverInfo>)          // number, code, name, team, team colour
 ├─ Timing(Vec<TimingDelta>)          // pos, gap, interval, laps, last/best, S1/S2/S3, in-pit
 ├─ Stints(Vec<StintDelta>)           // compound, tyre age, new/used
 ├─ Telemetry(Vec<CarSample>)         // t, driver, speed, throttle, brake, rpm, gear, drs
 ├─ Weather(WeatherSample)            // air, track, humidity, wind, rainfall
 ├─ TrackStatus(TrackStatus)          // Green|Yellow|SC|VSC|Red|Chequered
 ├─ RaceControl(Vec<Message>)
 ├─ Clock(SessionClock)               // elapsed/remaining, running flag
 └─ Link(LinkStatus)                  // Connecting|Live|Stale{since}|Reconnecting{attempt}
```

Events are plain `serde`-derived structs, constructed once by the decoder and moved (not cloned) through the channel.

### 2.6 Backpressure & Overload Policy

- Feed channel: bounded (`1024`). Producers use `send().await`; the telemetry path uses `try_send` and **drops the oldest sample batch** (telemetry is lossy by nature, timing is not).
- The app task drains the channel in a loop (`try_recv`) before marking dirty, so one render covers many messages.
- Telemetry history lives in fixed-capacity ring buffers (e.g., 600 samples ≈ 2.5 min at \~3.7 Hz per driver); memory is bounded by construction.

### 2.7 Decoupling Network from the 30–60 FPS Render Path

1. The render tick is an independent `tokio::time::interval` with `MissedTickBehavior::Skip`; it never awaits network.
2. `terminal.draw` runs on the app task and takes `&AppState` only. It performs no I/O, no parsing, and no locking.
3. Parsing runs in the source task (or `spawn_blocking` for large JSON bursts), so a 40 KB telemetry frame cannot delay a keypress.
4. If the socket stalls, the app task keeps ticking and redraws a `STALE 00:07` badge; the UI stays interactive (navigation, panel switch, quit).

### 2.8 Pause, Seek & Replay Semantics

- **Pause** freezes the *view*, not ingestion: the app stores `frozen: Option<Arc<RaceState>>`. Live state keeps updating underneath. **Resume** drops the frozen handle and jumps to live.
- **Replay mode** swaps `FeedSource` for `ReplaySource` with speed control (0.5×–8×) and seek by lap/time; same decoder, same UI.

---

## 3. UI/UX Design & Layout Specification (btop-style)

### 3.1 Visual Language

- **Borders:** rounded box-drawing (`╭ ─ ╮ │ ╰ ╯`) with the panel title inset in the top border, e.g. `╭─ ¹Timing ───╮`; the superscript digit is the hotkey that focuses that panel (btop idiom).
- **Density:** no empty padding rows; one text row per driver; numeric columns right-aligned with fixed widths.
- **Focus:** the focused panel border is drawn in the accent colour; others in dim grey.
- **Graphs:** Braille (2×4 dots per cell) for line graphs; block elements (`▁▂▃▄▅▆▇█`) for sparklines and bars.
- **Typography:** the terminal's monospace font only; bold for position/leader, dim for stale data, underline for the selected row.

### 3.2 Reference Layout (120×36)

```text
╭─ f1interm ─ Monza · RACE · Lap 23/53 ─ GREEN ─ LIVE ● ──────────────────────────────────────────────╮
│ AIR 24.1°  TRACK 38.6°  HUM 52%  WIND 2.1m/s  RAIN ─                          Remaining 0:41:07 │
╰─────────────────────────────────────────────────────────────────────────────────────────────╯
╭─ ¹Timing Tower ────────────────────────────────────╮╭─ ²Telemetry · VER ───────────────────╮
│ P  DRV  GAP      INT     S1   S2   S3    LAST    T   ││ SPD  312 km/h  ⢀⣀⣠⣤⣶⣿⣿⣶⣤⣀⡀⣠⣴⣾⣿⣷⣤   │
│ 1  VER  LEADER   ─       ▮▮   ▮▮   ▮     1:24.8  [M] ││ THR  100%      ▁▂▄▆██████▇▅▃▁▂▅███    │
│ 2  NOR  +1.204   +1.204  ▮▮   ▮    ▮▮    1:24.9  [M] ││ BRK  off       ▁▁▁▁▁▁▁▁▁▁▁▁███▁▁▁    │
│ 3  LEC  +3.890   +2.686  ▮    ▮▮   ▮▮    1:25.1  [H] ││ RPM  11 200    ⣀⣤⣶⣿⣿⣶⣤⣀⣤⣶⣿⣷⣤⣀⣠⣴     │
│ …                                                    ││ GEAR 7   DRS ON                       │
│ 20 SAR  +1 LAP   +4.1    ▮    ▮    ▮     1:27.3  [S] │╰────────────────────────────────────────╯
│                                                      │╭─ ³Compare · VER vs NOR ───────────────╮
│                                                      ││ Δ speed ⣀⣤⣶⣿ │ Δ time +0.031s          │
╰───────────────────────────────────────────────────────╯╰───────────────────────────────────────╯
╭─ ⁴Race Control ─────────────────────────────────────────────────────────────────────────────────╮
│ 14:21:07  CAR 16 (LEC) TIME 1:24.2 DELETED - TRACK LIMITS                                       │
╰────────────────────────────────────────────────────────────────────────────────────────────────╯
 j/k move · Tab panel · c compare · Space pause · / session · ? help · q quit
```

### 3.3 `ratatui` Layout Schema

Use nested `Layout` splits with `Constraint`s so the grid reflows on resize:

| Region | Direction | Constraints |
| --- | --- | --- |
| Root | Vertical | `Length(3)` header, `Min(10)` body, `Length(5)` race control (collapsible), `Length(1)` footer |
| Body | Horizontal | `Percentage(58)` timing tower, `Percentage(42)` right column |
| Right column | Vertical | `Percentage(55)` telemetry, `Percentage(45)` compare |

**Responsive tiers** (re-evaluated on every `Resize`):

- **≥ 140 cols:** full layout, with an optional track-map pane.
- **100–139 cols:** default layout above.
- **80–99 cols:** timing tower + telemetry only; compare opens as a modal overlay.
- **< 80×24:** compact timing list with a "terminal too small" hint; no graphs.

### 3.4 Colour Palette (truecolor, 24-bit)

Rendered with `Color::Rgb(r,g,b)`. Team colours are **sourced at runtime** from the feed's `team_colour` field when present, falling back to this table (approximate broadcast palette, kept in a single `theme.rs` so it can be updated each season):

| Team | Fallback hex |
| --- | --- |
| Ferrari | `#E8002D` |
| Red Bull Racing | `#3671C6` |
| Mercedes | `#27F4D2` |
| McLaren | `#FF8000` |
| Aston Martin | `#229971` |
| Alpine | `#0093CC` |
| Williams | `#64C4FF` |
| Racing Bulls | `#6692FF` |
| Haas | `#B6BABD` |
| Audi / Cadillac / new entries | from feed; else neutral `#AAAAAA` |

**UI chrome palette (dark default):**

| Role | Hex |
| --- | --- |
| Background | terminal default (transparent, like btop) |
| Border (unfocused) | `#4C566A` |
| Border (focused) / accent | `#E10600` (F1 red) |
| Primary text | `#D8DEE9` |
| Dim / stale | `#6B7280` |
| Overall-best / purple sector | `#B14FFF` |
| Personal best / green sector | `#3DDC84` |
| Slower / yellow sector | `#F5C542` |

**Tyre badges:** Soft `[S]` red `#FF3333`, Medium `[M]` yellow `#FFD12E`, Hard `[H]` white `#F0F0F0`, Intermediate `[I]` green `#39B54A`, Wet `[W]` blue `#2D8CFF`. Tyre age is shown dim after the badge (`[M]12`).

**Track status colours:** Green `#3DDC84`, Yellow `#F5C542`, SC/VSC `#FF9F1C` (with `SC`/`VSC` text), Red flag `#E10600` (header background pulses at 1 Hz, with a text label so it is not colour-only), Chequered black/white.

**Graceful degradation:** detect `COLORTERM=truecolor|24bit`; otherwise map to the nearest ANSI-256 colour; `NO_COLOR` and `--no-color` are honoured. A colour-blind theme swaps red/green deltas for blue/orange.

### 3.5 Module Specifications

#### Module 1 — Timing Tower (Leaderboard)

- **Columns:** `P` (position, with ▲/▼ change arrows for 3 s after a move), `DRV` (3-letter code in team colour with a 1-cell team stripe), `GAP` (to leader), `INT` (to car ahead), `S1 S2 S3` (sector times/bars coloured purple/green/yellow), `LAST`, `BEST`, tyre badge `[S]/[M]/[H]/[I]/[W]` + age, pit/`OUT` indicator, DRS dot.
- **Behaviour:** sorted by position; retired cars sink with a dim `OUT`; selected row underlined; pit-lane entry shows `PIT` with a running stop timer; lapped cars show `+1 LAP`.
- **Practice/Quali variants:** column set switches to best lap + gap-to-fastest, with the Q1/Q2/Q3 elimination line drawn as a divider.
- **Column auto-hide:** `BEST`, then `S1–S3`, then `INT` are dropped in that order as width shrinks.

#### Module 2 — Real-Time Telemetry Graphs

- Five channels for the selected driver: **Speed, Throttle, Brake, RPM, Gear**.
- Speed and RPM: Braille line graph (2×4 sub-cell resolution) in the driver's team colour. Throttle: block sparkline. Brake: binary block bar. Gear: step trace.
- Time window selectable (`+`/`-`): 15 s, 30 s, 60 s, or a last-lap overlay. The X axis is session time, not wall time, so pause/replay stay consistent.
- Implementation: `ratatui::widgets::Sparkline` for bar-style channels; a custom `BrailleGraph` widget (own `Widget` impl writing directly to `Buffer`) for line channels, since the built-in `Chart` is less dense.

#### Module 3 — Session & Track Status

- Header strip: session name, circuit, session type, lap `N/M` (race) or clock (practice/quali), link status (`LIVE ●`, `STALE`, `RECONNECTING`, `REPLAY 2×`).
- Weather: air/track temperature, humidity, wind speed/direction, rainfall indicator; short temperature sparklines.
- Track status badge with colour, plus sector-level yellow flags when available.
- Race control ticker in its own collapsible panel (`4`), newest first, with flag/penalty keyword highlighting.

#### Module 4 — Driver Focus / Comparison

- Two drivers (A and B) chosen with `c` (set A = highlighted, move, press `c` again for B).
- **Overlay mode:** both drivers' speed traces on one Braille graph in each team colour (teammates: second driver dashed/lighter).
- **Delta mode:** speed delta and cumulative time delta along the lap (distance-aligned when lap-relative position is available, otherwise time-aligned).
- **Table:** last/best lap, sector comparison, top speed, average throttle %, brake events, tyre/age.

### 3.6 Interaction Model

See §4.2 for the keymap. A persistent footer lists the most relevant keys for the focused panel; `?` opens a full help overlay.

---

## 4. Functional Requirements

### 4.1 Session Selection

| ID | Requirement |
| --- | --- |
| FR-1 | CLI: `f1interm --live` attaches to the current/next session; `--session <key>` or `--year 2026 --meeting monza --session race` selects a specific session. |
| FR-2 | `f1interm --replay <file\|session>` plays a recorded or historical session; `--speed 2.0` sets the initial speed. |
| FR-3 | With no arguments, an interactive menu lists the live session (if any) then recent sessions (year → meeting → session type: FP1/FP2/FP3, Sprint Qualifying, Sprint, Qualifying, Race). `j/k` navigate, `Enter` selects, `/` filters. |
| FR-4 | Session type changes layout defaults (race: gaps/intervals/tyres; qualifying: best lap/knock-out line; practice: best laps/long-run pace). |
| FR-5 | `--record <file>` writes the normalized event stream as NDJSON for later replay. |
| FR-6 | `--mock` runs a synthetic generator (no network) for development and demos. |

### 4.2 Keybindings

| Key | Action |
| --- | --- |
| `j` / `↓`, `k` / `↑` | Move driver selection |
| `g` / `G` | Jump to P1 / last |
| `Tab` / `Shift+Tab` | Cycle panel focus; `1`–`4` focus a panel directly |
| `Enter` | Set telemetry target to the highlighted driver |
| `c` | Pick comparison drivers (A then B); `C` clears |
| `Space` | Pause / resume view (ingestion continues) |
| `+` / `-` | Change graph time window; in replay `]` / `[` change speed |
| `←` / `→` (replay) | Seek ∓10 s; `Shift` −1 lap |
| `s` | Cycle sort (position / best lap / last lap / gap) |
| `t` | Toggle team-colour vs. monochrome |
| `/` | Session picker |
| `r` | Force reconnect |
| `?` | Help overlay |
| `q` / `Ctrl+C` | Quit (terminal always restored) |

Keybindings are remappable via `config.toml` (§4.5). Mouse support (click row, scroll) is optional and off by default.

### 4.3 Data Handling

| ID | Requirement |
| --- | --- |
| FR-7 | On startup, bootstrap state from REST (`sessions`, `drivers`, latest `position`/`intervals`/`stints`/`weather`) so the UI is populated before the first push message. |
| FR-8 | Merge incremental updates idempotently (last-write-wins by timestamp per driver/field); out-of-order events must not regress state. |
| FR-9 | Unknown JSON fields are ignored (`#[serde(default)]`, no `deny_unknown_fields`); missing fields yield `None` and render as `--`. |
| FR-10 | Telemetry history is kept in per-driver ring buffers with fixed capacity. |
| FR-11 | All timestamps are normalized to `chrono::DateTime<Utc>`; display uses session-local or UTC per config. |

### 4.4 Error Handling & Reconnection

| ID | Requirement |
| --- | --- |
| FR-12 | A `feed_supervisor` restarts the source on error with exponential backoff and jitter: 1 s → 2 s → 4 s … capped at 30 s; backoff resets after 30 s of healthy streaming. |
| FR-13 | Heartbeat/idle watchdog: if no frame arrives for `stale_after` (default 10 s), emit `Link(Stale)`; after `dead_after` (30 s) force a reconnect. |
| FR-14 | On reconnect, re-bootstrap from REST for the gap window, then resume the stream; the UI shows `RECONNECTING (attempt n)` and keeps the last good data dimmed. |
| FR-15 | HTTP 429/5xx: honour `Retry-After`, back off, and surface a one-line non-modal warning in the header. |
| FR-16 | Decode failures are counted and logged (to a log file, never stdout/stderr while the TUI runs); more than N consecutive failures flags the source as degraded. |
| FR-17 | A panic hook restores the terminal (leave alternate screen, disable raw mode, show cursor) before printing the panic report (`color-eyre`). |
| FR-18 | Graceful shutdown: on `q`/`SIGINT`/`SIGTERM`, cancel a `CancellationToken`, await tasks with a 1 s timeout, restore the terminal, exit 0. |

### 4.5 Configuration

`~/.config/f1interm/config.toml` (XDG; `%APPDATA%` on Windows): theme, keymap overrides, default session behaviour, graph window, units (metric/imperial), time zone, log level, optional `openf1_token`. CLI flags override config; environment variables override both for secrets.

---

## 5. Non-Functional Requirements

### 5.1 Performance

| ID | Requirement | How it is met |
| --- | --- | --- |
| NFR-1 | CPU < 2% of one average core in a live race (20 cars) | Event-driven redraw, dirty flag, diff-based `ratatui` buffer flush, no per-frame allocation in hot paths |
| NFR-2 | RSS < 30 MB | Bounded ring buffers, interned driver/team strings, no unbounded history; optional `--history-secs` |
| NFR-3 | ≥ 30 FPS when active, 60 FPS capable | 33 ms tick (configurable down to 16 ms); idle frames skipped entirely |
| NFR-4 | p99 input latency < 50 ms | `crossterm::event::EventStream` in `select!`; no blocking calls on the app task |
| NFR-5 | Parsing throughput ≥ 10× real-time peak (headroom for catch-up/replay) | `serde_json` on borrowed `&str` where possible; `spawn_blocking` for large JSON bursts |
| NFR-6 | Stripped binary < 8 MB | `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `strip = true`; trim `tokio`/`reqwest` features |

Performance is verified, not assumed: a `--bench-render` mode and `criterion` benches measure frame build time (target < 2 ms for a full 120×36 frame), and `tracing` spans / `tokio-console` measure channel queue depth.

### 5.2 Reliability & Safety

- **No shared mutable state on the hot path.** `AppState` is owned by one task; cross-task communication is exclusively via `mpsc` messages and `CancellationToken`. Where a shared handle is unavoidable (e.g., config hot reload), use `ArcSwap` or `tokio::sync::watch`, never a long-held `Mutex`.
- **`#![forbid(unsafe_code)]`** in the application crate.
- **Fault isolation:** a decoder panic is caught at the task boundary (`JoinHandle` result), logged, and the supervisor restarts the source; the UI continues.
- **Deterministic tests:** replay files plus an injectable clock make the whole pipeline testable without the network.
- **Testing strategy:** unit tests for parsers and reducers; property tests (`proptest`) for out-of-order/duplicate event merging; snapshot tests of rendered frames using `ratatui::backend::TestBackend` + `insta`; a 24 h replay-loop soak test tracking RSS.

### 5.3 Portability

| Platform | Terminals | Notes |
| --- | --- | --- |
| Linux | Alacritty, Kitty, Foot, WezTerm, GNOME Terminal | Primary target; Wayland and X11 |
| macOS | iTerm2, Ghostty, Terminal.app | Terminal.app lacks reliable truecolor → ANSI-256 fallback |
| Windows | Windows Terminal (ConPTY) | Legacy `conhost` unsupported; Braille glyph coverage depends on font |

- Rust edition 2021+, MSRV pinned and tested in CI.
- TLS via `rustls` (no OpenSSL dependency) for static, portable builds.
- Glyph fallback: `--ascii` mode replaces Braille/box-drawing with ASCII (`#`, `+`, `-`, `|`) for limited fonts.
- CI matrix: Linux/macOS/Windows × stable/MSRV; release artefacts for `x86_64` and `aarch64` on all three.

### 5.4 Security & Compliance

- Secrets only from env vars or a permissions-checked config file (`0600`); never logged.
- Dependency hygiene: `cargo audit` and `cargo deny` in CI.
- Respect upstream terms of service and rate limits (§8); clear attribution in `--about`.

### 5.5 Observability

`tracing` with a rolling file appender (`~/.local/state/f1interm/ferro.log`); `--log debug`; an in-app debug overlay (`F12`) showing FPS, queue depth, last-frame age, parse errors, and reconnect count.

---

## 6. Phased Implementation Roadmap

### Phase 1 — Scaffolding, Async Loop, Mock Data, Static Layout (Weeks 1–2)

**Goal:** a runnable skeleton that proves the architecture.

- Crate setup; `color-eyre`, `tracing`, `clap`.
- Terminal init/restore with panic hook; `tokio` runtime; `select!` loop over input + tick + channel.
- `FeedEvent` enum and `RaceState` reducer; `MockSource` emitting plausible 20-car timing + telemetry at realistic cadence.
- Static btop-style layout grid with placeholder widgets and responsive tiers.
- `TestBackend` snapshot tests for layout at 80×24, 120×36, 160×48.

**Exit criteria:** mock race runs 30 min at < 2% CPU; resize never panics; `q` always restores the terminal.

### Phase 2 — Live OpenF1 / WebSocket Integration & State Binding (Weeks 3–4)

**Goal:** real data end to end.

- `FeedSource` trait; `OpenF1Source` (REST bootstrap + live push) with auth-token support.
- Decoders → normalized events; idempotent merge logic (FR-8).
- Supervisor with backoff, watchdog, and stale/reconnect UI states (FR-12–FR-16).
- Session selection (CLI + interactive menu); `--record` / `--replay`.
- Bind the Timing Tower and Session/Track/Weather modules to real state.

**Exit criteria:** a full recorded race replays correctly; killing the network mid-session recovers automatically; live and replay paths produce identical state for the same input.

### Phase 3 — Advanced Widgets & Interaction (Weeks 5–6)

**Goal:** the signature visuals.

- Custom `BrailleGraph` widget; `Sparkline` channels; telemetry panel for Speed/Throttle/Brake/RPM/Gear with selectable time windows.
- Team colour mapping (runtime + fallback theme), tyre badges, sector colouring, position-change indicators.
- Interactive driver selection, panel focus, comparison module (overlay + delta), help overlay, config file and keymap overrides.
- Qualifying/practice layout variants.

**Exit criteria:** all four modules complete; keyboard-only operation covers every feature; frame build time < 2 ms at 120×36.

### Phase 4 — Polish, Profiling & Packaging (Weeks 7–8)

**Goal:** release quality.

- Profiling with `cargo flamegraph`, `tokio-console`, and `dhat`; tune allocations and redraw cadence to hit NFR-1/2.
- Accessibility (colour-blind theme, `NO_COLOR`, `--ascii`), terminal-compat testing matrix, Windows Terminal pass.
- Docs: README with asciinema/GIF, man page, `--help`, architecture notes.
- Packaging: `cargo install f1interm`, GitHub Releases (cross-built via `cargo-dist`), Homebrew tap, AUR package, optional Scoop manifest.
- Soak test, `cargo audit`, semver `v0.1.0` release.

**Exit criteria:** all NFR targets met on reference hardware; CI green on three OSes; install from crates.io works on a clean machine.

### Post-v1 Backlog

Track map (from `location` data), tyre-strategy timeline, pit-stop history, team-radio list, lap-time distribution charts, headless export (CSV/Parquet), custom panels, mouse support.

---

## 7. Key Rust Code Skeletons / Module Structure

### 7.1 Crate Dependencies (`Cargo.toml`, abridged)

```toml
[package]
name = "f1interm"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui     = "0.29"
crossterm   = { version = "0.28", features = ["event-stream"] }
tokio       = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync", "signal"] }
tokio-util  = "0.7"                       # CancellationToken
futures     = "0.3"                       # StreamExt for EventStream
async-trait = "0.1"
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-webpki-roots"] }
reqwest     = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
chrono      = { version = "0.4", features = ["serde"] }
clap        = { version = "4", features = ["derive", "env"] }
toml        = "0.8"
color-eyre  = "0.6"
tracing     = "0.1"
tracing-subscriber = "0.3"

[profile.release]
lto = "fat"
codegen-units = 1
strip = true
opt-level = 3
```

*Versions are indicative; pin to the latest compatible releases at project start.*

### 7.2 `src/` Layout

```text
f1interm/
├── Cargo.toml
├── src/
│   ├── main.rs          # entrypoint: CLI, runtime, terminal init/restore, panic hook
│   ├── cli.rs           # clap definitions, session selection args
│   ├── config.rs        # config.toml + keymap + env handling
│   ├── app.rs           # App struct, select! loop, key handling, UiState
│   ├── event.rs         # FeedEvent, Command, LinkStatus enums
│   ├── state.rs         # RaceState, DriverState, ring buffers, reducer (apply)
│   ├── network/
│   │   ├── mod.rs       # FeedSource trait, supervisor, backoff, watchdog
│   │   ├── openf1.rs    # REST bootstrap + live WS/MQTT source
│   │   ├── client.rs    # shared HTTP client, auth token, rate limits
│   │   ├── replay.rs    # NDJSON replay source with speed/seek
│   │   └── mock.rs      # synthetic generator
│   ├── model/           # serde DTOs mirroring upstream JSON (kept apart from state)
│   ├── ui/
│   │   ├── mod.rs       # draw(frame, &App): layout tiers, overlays
│   │   ├── theme.rs     # palette, team colours, tyre badges, ANSI-256 fallback
│   │   ├── timing.rs    # Module 1: timing tower
│   │   ├── telemetry.rs # Module 2: graphs
│   │   ├── status.rs    # Module 3: session/track/weather/race control
│   │   ├── compare.rs   # Module 4: driver comparison
│   │   └── widgets/     # BrailleGraph, TyreBadge, SectorBar, Gauge
│   └── util/            # ringbuf, time, formatting (gap/lap time), logging
└── tests/               # replay fixtures, snapshot tests, proptests
```

### 7.3 Core Types (`event.rs`, `state.rs`)

```rust
// event.rs — immutable messages crossing the channel boundary
#[derive(Debug, Clone)]
pub enum FeedEvent {
    Session(SessionMeta),
    Drivers(Vec<DriverInfo>),
    Timing(Vec<TimingDelta>),
    Stints(Vec<StintDelta>),
    Telemetry(Vec<CarSample>),
    Weather(WeatherSample),
    TrackStatus(TrackStatus),
    RaceControl(Vec<RaceControlMsg>),
    Clock(SessionClock),
    Link(LinkStatus),
}

#[derive(Debug, Clone)]
pub enum Command { Pause, Resume, SelectSession(SessionKey), Seek(Seek), Reconnect }

// state.rs — single-writer state, cheap snapshots via Arc
#[derive(Clone, Default)]
pub struct RaceState {
    pub session: Option<SessionMeta>,
    pub drivers: BTreeMap<DriverNo, DriverState>,
    pub order: Vec<DriverNo>,            // derived from positions
    pub weather: Option<WeatherSample>,
    pub track: TrackStatus,
    pub clock: SessionClock,
    pub link: LinkStatus,
    pub race_control: VecDeque<RaceControlMsg>,
}

impl RaceState {
    /// Pure reducer: idempotent, order-tolerant merge of one event.
    pub fn apply(&mut self, ev: FeedEvent) { /* match ev { ... } */ }
}
```

### 7.4 Network Trait & Supervisor (`network/mod.rs`)

```rust
#[async_trait::async_trait]
pub trait FeedSource: Send + 'static {
    /// Run until error or cancellation, pushing normalized events.
    async fn run(&mut self, tx: mpsc::Sender<FeedEvent>, cancel: CancellationToken)
        -> color_eyre::Result<()>;
}

pub async fn supervise<F, S>(mut make: F, tx: mpsc::Sender<FeedEvent>, cancel: CancellationToken)
where
    F: FnMut() -> S + Send + 'static,
    S: FeedSource,
{
    let mut attempt = 0u32;
    while !cancel.is_cancelled() {
        let _ = tx.send(FeedEvent::Link(LinkStatus::Connecting { attempt })).await;
        let started = Instant::now();
        let mut src = make();
        let _ = src.run(tx.clone(), cancel.child_token()).await;   // Ok or Err: either way, reconnect
        if cancel.is_cancelled() { break; }
        if started.elapsed() > Duration::from_secs(30) { attempt = 0; }  // healthy run resets backoff
        attempt += 1;
        let delay = backoff(attempt);                                     // 1s,2s,4s… ≤ 30s + jitter
        let _ = tx.send(FeedEvent::Link(LinkStatus::Reconnecting { attempt, retry_in: delay })).await;
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = cancel.cancelled() => break,
        }
    }
}
```

### 7.5 Main Event Loop (`main.rs` + `app.rs`)

```rust
// main.rs
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;                 // panic/error hooks (also restore the terminal)
    let cli = Cli::parse();
    let cfg = Config::load(&cli)?;
    let mut terminal = ratatui::init();     // raw mode + alternate screen
    let result = App::new(cli, cfg).run(&mut terminal).await;
    ratatui::restore();                     // always runs, even on Err
    result
}
```

```rust
// app.rs
pub struct App {
    state: Arc<RaceState>,
    frozen: Option<Arc<RaceState>>,         // set while paused
    ui: UiState,                            // selection, focus, windows, overlays
    dirty: bool,
    quit: bool,
    cfg: Config,
}

impl App {
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let (ev_tx, mut ev_rx) = mpsc::channel::<FeedEvent>(1024);
        let (cmd_tx, cmd_rx)   = mpsc::channel::<Command>(32);
        let cancel = CancellationToken::new();

        let net = tokio::spawn(network::start(self.cfg.source(), ev_tx, cmd_rx, cancel.clone()));

        let mut keys   = EventStream::new();                                // async crossterm events
        let mut render = tokio::time::interval(Duration::from_millis(33));  // ~30 FPS
        render.set_missed_tick_behavior(MissedTickBehavior::Skip);
        self.dirty = true;

        while !self.quit {
            tokio::select! {
                // 1) frame tick: draw only if something changed
                _ = render.tick() => {
                    if self.dirty {
                        terminal.draw(|f| ui::draw(f, &self))?;
                        self.dirty = false;
                    }
                }

                // 2) terminal input
                Some(Ok(ev)) = keys.next() => match ev {
                    Event::Key(k) if k.kind == KeyEventKind::Press => {
                        self.on_key(k, &cmd_tx).await;
                        self.dirty = true;
                    }
                    Event::Resize(_, _) => self.dirty = true,
                    _ => {}
                },

                // 3) feed events: apply, then drain the burst before the next draw
                Some(first) = ev_rx.recv() => {
                    let st = Arc::make_mut(&mut self.state);   // copy-on-write if a snapshot exists
                    st.apply(first);
                    while let Ok(more) = ev_rx.try_recv() { st.apply(more); }
                    if self.frozen.is_none() { self.dirty = true; }   // paused view stays still
                }
            }
        }

        cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(1), net).await;
        Ok(())
    }

    /// The view reads the frozen snapshot while paused, the live state otherwise.
    pub fn view(&self) -> &RaceState { self.frozen.as_deref().unwrap_or(&self.state) }

    fn toggle_pause(&mut self) {
        self.frozen = match self.frozen.take() {
            Some(_) => None,
            None => Some(self.state.clone()),
        };
    }
}
```

```rust
// ui/mod.rs
pub fn draw(f: &mut Frame, app: &App) {
    let s = app.view();
    let [header, body, rc, footer] = Layout::vertical([
        Constraint::Length(3), Constraint::Min(10), Constraint::Length(5), Constraint::Length(1),
    ]).areas(f.area());
    let [left, right] = Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).areas(body);
    let [tele, cmp]   = Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(right);

    status::render_header(f, header, s, &app.ui);
    timing::render(f, left, s, &app.ui);
    telemetry::render(f, tele, s, &app.ui);
    compare::render(f, cmp, s, &app.ui);
    status::render_race_control(f, rc, s, &app.ui);
    status::render_footer(f, footer, &app.ui);
}
```

*Notes on the skeleton:* `ratatui::init()` / `ratatui::restore()` and the `Layout::areas` helpers need a recent `ratatui` (0.28+). In raw mode Ctrl+C arrives as a key event, so `q`/`Ctrl+C` handling lives in `on_key`; external `SIGINT`/`SIGTERM` should be wired as an extra `select!` branch (a signal stream created once outside the loop, with `tokio::signal::unix` on Unix). Sketches are illustrative and will need small adjustments to compile against the pinned crate versions.

---

## 8. Risks, Assumptions & Open Questions

| # | Risk / Assumption | Mitigation |
| --- | --- | --- |
| R1 | **Upstream access terms.** OpenF1 live push may require authentication or a paid tier; if it is unavailable, v1 falls back to REST polling and replay. | Confirm terms before Phase 2; keep sources behind `FeedSource`; ship REST/replay as the guaranteed baseline; no data redistribution. |
| R2 | **Schema drift** between seasons or mid-season. | Tolerant decoding (`serde(default)`), recorded-session regression fixtures, per-season theme file. |
| R3 | **Telemetry sample rate and latency** (a few Hz, with upstream delay) limits smoothness. | Interpolate visually only (never invent data); show feed delay in the debug overlay. |
| R4 | **Terminal glyph support** (Braille, box-drawing) varies by font and OS. | `--ascii` fallback; documented recommended fonts; glyph self-test command. |
| R5 | **Feed-time vs. wall-time** confusion in pause/replay. | All graphs keyed on session time; one `SessionClock` abstraction. |
| R6 | **New grid entries and livery changes** between seasons. | Colours come from the feed first; fallback table isolated in `theme.rs`. |

**Open questions:** Should v1 fall back to REST polling when live push is unavailable? Is a track-map pane in scope for v1 or v1.1? Should recorded sessions be shareable (format stability guarantees)? Is an 80×24 compact mode an acceptable minimum?

---

## Appendix A — Requirement Traceability (summary)

| Area | IDs |
| --- | --- |
| Session selection | FR-1 … FR-6 |
| Data handling | FR-7 … FR-11 |
| Resilience | FR-12 … FR-18 |
| Performance | NFR-1 … NFR-6 |
| UI modules | §3.5 (Modules 1–4) |

## Appendix B — Definition of Done (v0.1.0)

- All four modules implemented and keyboard-operable.
- Live, replay, mock, and record modes working; automatic reconnection verified by a network-kill test.
- NFR-1 … NFR-6 measured and documented on reference hardware.
- CI green on Linux, macOS, Windows; `cargo audit` clean.
- README, man page, and a demo recording published; `cargo install f1interm` succeeds.
