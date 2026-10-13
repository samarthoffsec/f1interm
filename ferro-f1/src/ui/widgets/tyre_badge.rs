/// Tyre compound badge showing `[S]`, `[M]`, `[H]`, `[I]`, `[W]` with optional age.
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
    widgets::{Paragraph, Block},
};

/// A tyre compound badge showing `[S]`, `[M]`, `[H]`, `[I]`, `[W]` with optional age.
#[derive(Debug, Clone)]
pub struct TyreBadge {
    compound: String,
    age: u16,
    max_age: u16,
    style: Style,
}

impl TyreBadge {
    pub fn new(compound: &str, age: u16, max_age: u16, style: Style) -> Self {
        let compound_upper = compound.to_uppercase();
        Self {
            compound: compound_upper,
            age,
            max_age,
            style,
        }
    }

    fn compound_colour(&self) -> (u8, u8, u8) {
        match self.compound.as_str() {
            "S" => (255, 51, 51),    // Soft - red
            "M" => (255, 209, 46),  // Medium - yellow
            "H" => (240, 240, 240), // Hard - white
            "I" => (57, 181, 74),   // Intermediate - green
            "W" => (45, 140, 255),  // Wet - blue
            _ => (170, 170, 170),   // Neutral/Unknown
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let usable_width = area.width.max(1);
        let usable_height = area.height.max(1);

        if usable_width < 3 {
            return;
        }

        let (r, g, b) = self.compound_colour();
        let colour = Color::Rgb(r, g, b);
        let compound_char = self.compound.chars().next().unwrap_or('?');
        let age_str = if self.age > 0 { format!("{}", self.age) } else { String::new() };

        // Build badge: [CHAR]age or [CHAR] if no age
        let display_age = if self.age > self.max_age {
            format!("{}*", self.max_age)
        } else {
            age_str
        };
        let final_text = if !display_age.is_empty() {
            format!("[{}]{}", compound_char, display_age)
        } else {
            format!("[{}]", compound_char)
        };

        let paragraph = Paragraph::new(final_text)
            .style(self.style.fg(colour).bg(self.style.bg))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

impl Default for TyreBadge {
    fn default() -> Self {
        Self {
            compound: "S".to_string(),
            age: 0,
            max_age: 20,
            style: ratatui::style::Style::default(),
        }
    }
}