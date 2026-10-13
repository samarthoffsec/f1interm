//! f1interm User Interface modules.
//! Modules: theme, widgets, layout, rendering.

/// Colour palette, team colours, and UI theming.
pub mod theme;
use self::theme::{UiColour, TyreColour, team_colour_for_code, apply_colour_blind, ui_colour_role, tyre_colour_role};

/// Re-usable UI widgets: Braille graphs, sparklines, badges, etc.
pub mod widgets;
use self::widgets::{BrailleGraph, Sparkline, TyreBadge, SectorBar};

// Re-export types from state for UI access
use crate::state::RaceState;

/// Main UI draw function.
///
/// Renders the full TUI interface with layout tiers:
/// - Header strip (session info, weather, link status)
/// - Body: timing tower (left) + right column (telemetry + compare)
/// - Race control panel (collapsible)
/// - Footer
pub fn draw(
    f: &mut ratatui::Frame,
    app: &crate::App,
    no_color: bool,
    ascii_mode: bool,
) -> color_eyre::Result<()> {
    let s = app.view();
    let ui = &app.ui;

    // Helper lambda to get colour respecting no_color and colour_blind
    let get_colour = |role: &str| -> ratatui::style::Color {
        let ui_col = ui_colour_role(role, ui.colour_blind, no_color);
        match ui_col {
            UiColour::Rgb(r, g, b) => ratatui::style::Color::Rgb(r, g, b),
            UiColour::Ansi8(index) => ratatui::style::Color::Index(8 + index),
        }
    };

    // Header strip: 3 rows tall
    let header_height = 3f32;
    let header_area = f.area().height.max(1);
    let header_chunk = ratatui::layout::Constraint::Length(header_height as u16);
    let chunks = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(3),
        ratatui::layout::Constraint::Min(10),
        ratatui::layout::Constraint::Length(5),
        ratatui::layout::Constraint::Length(1),
    ])
    .areas(f.area());

    let header = chunks[0];
    let body = chunks[1];
    let race_control = chunks[2];
    let footer = chunks[3];

    // ----- Header -----
    // Session name, circuit, session type, lap chart, weather, link status
    let session_text = s.session
        .as_ref()
        .map(|s| format!("{} · {} · {} · Lap {}/{}", s.name, s.circuit, match s.session_type { event::SessionType::Race => "RACE", event::SessionType::Qualifying => "QUALIFYING", event::SessionType::Practice => "PRACTICE" }, s.current_lap.max(0), s.total_laps.max(0)))
        .unwrap_or_default();

    let link_text = match s.link {
        event::LinkStatus::Live => "LIVE ●",
        event::LinkStatus::Stale { .. } => "STALE",
        event::LinkStatus::Reconnecting { .. } => "RECONNECTING",
        event::LinkStatus::Disconnected => "DISCONNECTED",
        event::LinkStatus::Connecting { .. } => "CONNECTING",
    };

    let link_colour = get_colour("border_focused"); // reuse accent colour

    // Weather info
    let weather_text = s.weather
        .as_ref()
        .map(|w| format!("AIR {:.1}° TRACK {:.1}° HUM {}% WIND {:.1}m/s RAIN {:.1}%", w.air_temp, w.track_temp, w.humidity, w.wind_speed, w.rainfall))
        .unwrap_or_default();

    // Combine header texts
    let header_text = format!("{} · {} · {} · {} {}", session_text, link_text, weather_text, "REMAINING {:02}:{:02}:{:02}", 0, 0, 0); // placeholder remaining

    let header_paragraph = ratatui::widgets::Paragraph::new(header_text)
        .style(
            ratatui::style::Style::new()
                .fg(get_colour("primary_text"))
                .bg(get_colour("background")),
        );
    f.render_widget(header_paragraph, header);

    // ----- Body: split left (timing tower) and right (telemetry + compare) -----
    let [left, right] = ratatui::layout::Layout::horizontal([
        ratatui::layout::Constraint::Percentage(58),
        ratatui::layout::Constraint::Percentage(42),
    ])
    .areas(body);

    // ----- Left: Timing Tower -----
    let [tele, cmp] = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Percentage(55),
        ratatui::layout::Constraint::Percentage(45),
    ])
    .areas(right);

    // Render timing tower on the left
    // We'll render a simplified version - just the driver list with key info
    // In a full implementation, this would use the timing module

    // For now, render a placeholder noting the panel
    let timing_text = if ascii_mode {
        "TIMING TOWER (ASCII MODE)".to_string()
    } else {
        "Timing Tower".to_string()
    };
    let timing_para = ratatui::widgets::Paragraph::new(timing_text)
        .style(
            ratatui::style::Style::new()
                .fg(get_colour("primary_text"))
                .bg(get_colour("background")),
        );
    f.render_widget(timing_para, left);

    // ----- Right: Telemetry (top) + Compare (bottom) -----
    // Telemetry panel
    let telemetry_para = ratatui::widgets::Paragraph::new("Telemetry")
        .style(
            ratatui::style::Style::new()
                .fg(get_colour("primary_text"))
                .bg(get_colour("background")),
        );
    f.render_widget(telemetry_para, tele);

    // Compare panel
    let compare_para = ratatui::widgets::Paragraph::new("Comparison")
        .style(
            ratatui::style::Style::new()
                .fg(get_colour("primary_text"))
                .bg(get_colour("background")),
        );
    f.render_widget(compare_para, cmp);

    // ----- Race Control panel -----
    let rc_text = if ascii_mode {
        "RACE CONTROL (ASCII)".to_string()
    } else {
        "Race Control".to_string()
    };
    let rc_para = ratatui::widgets::Paragraph::new(rc_text)
        .style(
            ratatui::style::Style::new()
                .fg(get_colour("primary_text"))
                .bg(get_colour("background")),
        );
    f.render_widget(rc_para, race_control);

    // ----- Footer -----
    let footer_text = "j/k move · Tab panel · c compare · Space pause · / session · ? help · q quit";
    let footer_para = ratatui::widgets::Paragraph::new(footer_text)
        .style(
            ratatui::style::Style::new()
                .fg(get_colour("dim_stale"))
                .bg(get_colour("background")),
        );
    f.render_widget(footer_para, footer);

    Ok(())
}