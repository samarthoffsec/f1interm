use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
    widgets::{Paragraph, Block},
};

/** A tyre compound badge showing `[S]`, `[M]`, `[H]`, `[I]`, `[W]` with optional age.
 *
 * Renders like: `[M]12` where M is the compound and 12 is the age in laps.
 * The compound letter is shown in the tyre's colour; the age is dim.
 *
 * In ASCII mode, the bracket pattern is simplified to `(M)12` for monospace compatibility.
 */
#[derive(Debug, Clone)]
pub struct TyreBadge {
    compound: String,
    age: u16,
    max_age: u16,
    style: Style,
    ascii_mode: bool,
}

impl TyreBadge {
    /// Creates a new tyre badge.
    /// `compound` should be one of: "S", "M", "H", "I", "W" (case-insensitive)
    /// `age` is the current tyre age in laps
    /// `max_age` is the maximum laps before pit stop
    /// `ascii_mode` renders simplified brackets in ASCII mode
    pub fn new(compound: &str, age: u16, max_age: u16, style: Style, ascii_mode: bool) -> Self {
        let compound_upper = compound.to_uppercase();
        Self {
            compound: compound_upper,
            age,
            max_age,
            style,
            ascii_mode,
        }
    }

    /// Returns the colour for this compound (hardcoded fallback palette).
    fn compound_colour(&self) -> (u8, u8, u8) {
        match self.compound.as_str() {
            "S" => (255, 51, 51),    // Soft - red #FF3333
            "M" => (255, 209, 46),  // Medium - yellow #FFD12E
            "H" => (240, 240, 240), // Hard - white #F0F0F0
            "I" => (57, 181, 74),   // Intermediate - green #39B54A
            "W" => (45, 140, 255),  // Wet - blue #2D8CFF
            _ => (170, 170, 170),   // Neutral/Unknown - #AAAAAA
        }
    }

    /// Renders the badge into the given area.
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let usable_width = area.width.max(1);
        let usable_height = area.height.max(1);

        if usable_width < 3 {
            return;
        }

        let (r, g, b) = self.compound_colour();
        let colour = Color::Rgb(r, g, b);
        let compound_char = self.compound.chars().next().unwrap_or('?');
        let age_str = if self.age > 0 {
            format!("{}", self.age)
        } else {
            String::new()
        };

        // Build badge: in ASCII mode use (CHAR)age, otherwise [CHAR]age
        let (badge_brackets, open_bracket) = if self.ascii_mode {
            ("(", compound_char)
        } else {
            ("[", compound_char)
        };

        let display_age = if self.age > self.max_age {
            format!("{}*", self.max_age)
        } else {
            age_str.clone()
        };

        let final_text = if !display_age.is_empty() {
            format!("{}{}{}", open_bracket, compound_char, display_age)
        } else {
            format!("{}{}", open_bracket, compound_char)
        };

        let paragraph = Paragraph::new(final_text)
            .style(self.style.fg(colour).bg(self.style.bg));
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
            ascii_mode: false,
        }
    }
}