//! Widgets for the f1interm TUI.
//! Custom widgets for Braille graphs, sparklines, tyre badges, and sector bars.

mod braille_graph;
mod sparkline;
mod tyre_badge;
mod sector_bar;

pub use braille_graph::BrailleGraph;
pub use sparkline::Sparkline;
pub use tyre_badge::TyreBadge;
pub use sector_bar::SectorBar;