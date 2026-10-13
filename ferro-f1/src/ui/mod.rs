//! UI modules for f1interm.
//! Modules: theme, widgets, layout, rendering.

pub mod theme;
use self::theme::{UiColour, TyreColour, team_colour_for_code, apply_colour_blind};

pub mod widgets;
use self::widgets::{BrailleGraph, Sparkline, TyreBadge, SectorBar};

// Re-export RaceState for UI access
use crate::state::RaceState;