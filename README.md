# f1interm — Formula 1 Real-Time Terminal Dashboard

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![TUI](https://img.shields.io/badge/TUI-ratatui%20%2B%20crossterm-brightgreen.svg)]

A btop-style terminal dashboard for Formula 1 data, written in Rust. Shows live timing, telemetry, session status, and driver comparison in a dense, keyboard-driven interface.

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Quick Start](#quick-start)
4. [CLI Arguments](#cli-arguments)
5. [Modes of Operation](#modes-of-operation)
   - [Mock Mode](#mock-mode)
   - [Live Mode (Phase 2)](#live-mode-phase-2)
   - [Replay Mode](#replay-mode)
   - [Record Mode](#record-mode)
6. [Keybindings](#keybindings)
7. [Configuration](#configuration)
8. [Building from Source](#building-from-source)
9. [Running the Tests](#running-tests)
10. [License](#license)

---

## Overview

`f1interm` (pronounced "ferro-f-one") is a single-binary, keyboard-driven terminal application that displays Formula 1 session data in a dense, btop-style dashboard. It is written in Rust using `ratatui` + `crossterm` + `tokio`, designed to run comfortably in a tmux pane next to your editor during a race weekend.

The project is structured in phases. **Phase 1** (current) provides a runnable skeleton with mock data, proving the architecture. Subsequent phases add live OpenF1 integration, advanced widgets, and packaging.

**Target audiences:**
- F1 enthusiasts wanting timing-screen depth without a browser
- Developers/hackers wanting a reference for async Rust + TUI + live feeds
- Terminal power users who value keyboard-first, low-overhead tools

**Design goals:**
- **Low latency:** data-to-cell under 100 ms (excluding upstream feed delay)
- **Zero bloat:** one static binary, no runtime, no browser engine, small audited dependency tree
- **High visual fidelity:** truecolor team palettes, Braille/block-character graphs, box-drawing panels, btop-inspired look
- **Never freeze:** network I/O, parsing, and rendering are strictly decoupled
- **Honest data:** every panel shows a staleness indicator; missing data renders as `--`

---

## Features (Phase 1)

| Feature | Status |
| --- | --- |
| Mock race generator (20 cars, realistic cadence) | ✅ |
| Static btop-style layout grid with responsive tiers | ✅ |
| Timing tower with driver positions, gaps, sector times, tyre badges | ✅ |
| Telemetry graphs: Speed, Throttle, Brake, RPM, Gear | ✅ |
| Session/track/weather status header | ✅ |
| Race control panel with collapsible list | ✅ |
| Driver comparison module | ✅ |
| Keybindings: j/k navigation, Tab panel cycle, Space pause, q quit | ✅ |
| Colour-blind theme swap, ASCII mode fallback | ✅ |
| `color-eyre` panic hook with terminal restore | ✅ |
| Configuration via CLI flags, env vars, and `~/.config/f1interm/config.toml` | ✅ |
| Bounded channel backpressure, fixed-capacity ring buffers | ✅ |
| Idempotent state reducer (order-tolerant merge) | ✅ |

---

## Quick Start

### Binary release (preferred)

If you just want to run the app, download a prebuilt binary from the [GitHub Releases page](https://github.com/your-org/f1interm/releases) and make it executable:

```bash
chmod +x f1interm-x86_64-unknown-linux-gnu
./f1interm
```

### Building from source

```bash
# Clone the repo
git clone https://github.com/your-org/f1interm.git
cd f1interm

# Build in release mode (produces a static binary ~5-8 MB)
cargo build --release

# The binary is at target/release/ferro-f1
```

### Run with mock data (no network needed)

```bash
# Using cargo run
cargo run -- --mock

# Or using the built binary
./target/release/ferro-f1 --mock
```

### Run with live/OpenF1 (Phase 2+, requires token)

```bash
OPENF1_TOKEN=your_token ./target/release/ferro-f1 --live
```

---

## CLI Arguments

```
Usage: ferro-f1 [OPTIONS]

Options:
  -l, --live       Attach to the current/live session (Phase 2: OpenF1)
  -m, --mock       Run synthetic mock generator (default for Phase 1)
  -q, --quit       (Phase 3) Exit after a short demo; restore terminal
  -n, --no-color   Disable truecolor output; use ANSI-256 palette
  -a, --ascii      Replace Braille/box-drawing with ASCII chars (#, +, -, |)
  -h, --help       Print help
```

**Environment variables:**

| Variable | Effect |
| --- | --- |
| `OPENF1_TOKEN` | Auth token for OpenF1 live push (Phase 2) |
| `NO_COLOR` | Honoured; equivalent to `--no-color` |
| `F1INTERM_ASCII` | Equivalent to `--ascii` |
| `RUST_LOG` | Tracing log level (default: `info`) |
| `F1INTERM_THEME` | Path to custom theme TOML (optional) |

---

## Modes of Operation

### Mock Mode (`--mock`)

Runs a synthetic generator that emits plausible 20-car timing + telemetry at realistic cadence. Useful for development, demos, and verifying the TUI loop without needing network access.

```bash
cargo run -- --mock
```

The mock generates:
- 20 cars with team assignments and colours
- Timing deltas (gap/interval to leader)
- Sector times (S1, S2, S3) with bars
- Weather sample (air/track temp, humidity, wind, rainfall)
- Track status (green/yellow/SC/VSC/Red/Checkered)
- Race control messages (pit limits, penalties)
- Lap progress (current lap, total laps)
- Session clock (elapsed/remaining)

### Live Mode (`--live`)

Phase 2+: Attaches to the current session via OpenF1 live push (WebSocket/MQTT). Requires `OPENF1_TOKEN` env var. Bootstrap from REST endpoints, then maintain connection with exponential-backoff reconnect.

### Replay Mode (`--replay <file>`)

Phase 3+: Plays a recorded NDJSON session file, emitted at original cadence × speed control. See the [Roadmap](#roadmap) for details.

```bash
cargo run -- --replay session.ndjson
```

### Record Mode (`--record <file>`)

Phase 3+: Writes the normalized event stream as NDJSON for later replay.

```bash
cargo run -- --record recording.ndjson
```

---

## Keybindings

All keys are pressed in the terminal; the app is always in "focus follows keyboard" mode. The footer line at the bottom of the screen shows the most relevant keys for the current panel.

| Key | Action |
| --- | --- |
| `j` / `↓` | Move driver selection down |
| `k` / `↑` | Move driver selection up |
| `g` / `G` | Jump to P1 / last position |
| `Tab` / `Shift+Tab` | Cycle panel focus (Timing → Telemetry → Status → Compare → repeat) |
| `1`–`4` | Focus panel directly (Timing=1, Telemetry=2, Status=3, Compare=4) |
| `Enter` | Set telemetry target to highlighted driver |
| `c` | Select comparison driver A (press again for B); `C` (Shift+C) clears |
| `Space` | Pause / resume view (ingestion continues underneath) |
| `+` / `=` | Increase graph time window (15s, 30s, 60s, …) |
| `-` | Decrease graph time window |
| `s` | Cycle sort order: position → best lap → last lap → gap → position |
| `t` | Toggle colour-blind mode (red/green deltas → blue/orange) |
| `z` | Toggle ASCII mode (Braille/box-drawing → `#+|-` chars) |
| `/` | Open session picker / filter menu |
| `r` | Force reconnect (Phase 2+) |
| `?` | Help overlay (full keymap) |
| `q` / `Ctrl+C` | Quit (terminal always restored to original state) |

**Persistent footer** (example):
```
j/k move · Tab panel · c compare · Space pause · / session · ? help · q quit
```

---

## Configuration

Configuration is loaded in this order (later overrides earlier):

1. **Defaults** built into `Config::from_env()`
2. **Environment variables** (`OPENF1_TOKEN`, `NO_COLOR`, `F1INTERM_ASCII`, `RUST_LOG`)
3. **Config file** `~/.config/f1interm/config.toml` (XDG Base Directory)
4. **CLI flags** (override everything)

### Config file location

`~/.config/f1interm/config.toml`

If the XDG directory is unavailable, the app falls back to `f1interm_config.toml` in the current working directory.

### Example config.toml

```toml
no_color = false
ascii_mode = false
units = "metric"
timezone = "UTC"
log_level = "debug"

[telemetry]
graph_window_secs = 30

[colour_blind]
enabled = false
```

### Config options

| Key | Default | Description |
| --- | --- | --- |
| `openf1_token` | `` | Auth token for OpenF1 live push |
| `default_session` | `None` | Pre-select a session key at startup (`"year:meeting:session_type"`) |
| `graph_window_secs` | `60` | Telemetry graph time window in seconds |
| `units` | `"metric"` | `"metric"` or `"imperial"` |
| `timezone` | `"UTC"` | Display timezone for timestamps |
| `log_level` | `"info"` | Tracing log level |
| `no_color` | `false` | Disable truecolor |
| `ascii_mode` | `false` | Replace Braille/box-drawing with ASCII |

---

## Building from Source

### Prerequisites

- Rust 1.75 or later
- A terminal that supports truecolor (Alacritty, Kitty, Foot, WezTerm, iTerm2, Ghostty) or ANSI-256 fallback
- Optional: `direnv` or `~/.config/f1interm/` for config

### Compile

```bash
# Debug build (faster iterates, binary ~2-3 MB)
cargo build

# Release build (optimized, static binary)
cargo build --release
```

The release binary is at `target/release/ferro-f1`.

### Cross-compilation

The project uses `cargo-dist` (future) for multi-arch releases. Currently, `cargo build --release` produces an `x86_64` or `aarch64` binary for the host platform.

### Running tests

```bash
cargo test
```

Snapshot tests validate the TUI layout at 80×24, 120×36, and 160×48 using `ratatui::backend::TestBackend`.

---

## Roadmap (Phased Implementation)

| Phase | Goal | Exit Criteria |
| --- | --- | --- |
| **Phase 1** (complete) | Scaffolding, async loop, mock data, static layout | Mock race runs 30 min at <2% CPU; resize never panics; `q` always restores terminal |
| **Phase 2** | Live OpenF1/WebSocket integration & state binding | Full recorded race replays correctly; network-drop auto-recovery; live & replay produce identical state |
| **Phase 3** | Advanced widgets & interaction | All four modules complete; keyboard-only operation covers every feature; frame build < 2 ms at 120×36 |
| **Phase 4** | Polish, profiling & packaging | All NFR targets met on reference hardware; CI green on Linux/macOS/Windows; `cargo install f1interm` succeeds |

**Post-v1 backlog:** Track map, tyre-strategy timeline, pit-stop history, team-radio list, lap-time distribution charts, headless CSV/Parquet export, custom panels, mouse support.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](./LICENSE) file for details.

---

## Contributing

1. Fork the repo and create a feature branch (`git checkout -b feature/foo`)
2. Make your changes, following the existing code style
3. Run `cargo test` and `cargo clippy` to verify
4. Commit with a descriptive message
5. Push and open a Pull Request

**Development checklist:**
- [ ] Add or update unit/integration tests
- [ ] Add snapshot tests for new/layout changes
- [ ] Run `cargo audit` and `cargo deny` for vulnerability checks
- [ ] Ensure `cargo clippy --all-targets` passes
- [ ] Update this README if API or CLI changes

---

## Contact & Support

- **Issues:** [GitHub Issues](https://github.com/your-org/f1interm/issues)
- **Discussion:** [GitHub Discussions](https://github.com/your-org/f1interm/discussions)
- **Author:** Principal Systems Architect – Rust / TUI design

---

*Enjoy the race, right from your terminal.* 🏎️💨