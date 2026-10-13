//! f1interm User Interface modules.
//! Modules: theme, widgets, layout, rendering.

/// Colour palette, team colours, and UI theming.
pub mod theme;
use self::theme::{UiColour, TyreColour, team_colour_for_code, apply_colour_blind};

/// Re-usable UI widgets: Braille graphs, sparklines, badges, etc.
pub mod widgets;
use self::widgets::{BrailleGraph, Sparkline, TyreBadge, SectorBar};

// Re-export types from state for UI access
use crate::state::RaceState;