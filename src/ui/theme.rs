/// Team colours: runtime from feed `team_colour`, fallback for when feed lacks it.
/// Keep in a single file so it can be updated each season.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeamColour(pub u8, pub u8, pub u8);

impl TeamColour {
    pub const FERRARI: Self = Self(232, 0, 45);
    pub const RED_BULL: Self = Self(54, 113, 198);
    pub const MERCEDES: Self = Self(39, 244, 210);
    pub const MCLEARAN: Self = Self(255, 128, 0);
    pub const ASTON_MARTIN: Self = Self(34, 153, 113);
    pub const ALPINE: Self = Self(0, 147, 204);
    pub const WILLIAMS: Self = Self(100, 196, 255);
    pub const RACING_BULLS: Self = Self(102, 146, 255);
    pub const HAAS: Self = Self(182, 186, 189);
    pub const NEUTRAL: Self = Self(170, 170, 170);

    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "ferrari" | "scuderia" => Self::FERRARI,
            "red bull" | "red bull racing" => Self::RED_BULL,
            "mercedes" => Self::MERCEDES,
            "mclaren" | "mcLaren" => Self::MCLEARAN,
            "aston martin" | "aston" => Self::ASTON_MARTIN,
            "alpine" => Self::ALPINE,
            "williams" => Self::WILLIAMS,
            "racing bulls" | "rb" => Self::RACING_BULLS,
            "haas" => Self::HAAS,
            _ => Self::NEUTRAL,
        }
    }

    pub fn as_rgb(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }
}

/// UI chrome palette (dark default, 24-bit truecolor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiColour(pub u8, pub u8, pub u8);

impl UiColour {
    pub const BACKGROUND: Self = Self(0, 0, 0);       // terminal default (transparent like btop)
    pub const BORDER_UNFOCUSED: Self = Self(76, 86, 106);   // #4C566A
    pub const BORDER_FOCUSED: Self = Self(225, 6, 0);       // #E10600 F1 red
    pub const PRIMARY_TEXT: Self = Self(216, 222, 233);     // #D8DEE9
    pub const DIM_STALE: Self = Self(107, 114, 128);       // #6B7280
    pub const PURPLE_SECTOR: Self = Self(177, 79, 255);    // #B14FFF overall-best
    pub const PERSONAL_BEST: Self = Self(61, 220, 132);    // #3DDC84 personal-best
    pub const SLOWER: Self = Self(245, 197, 66);          // #F5C542 slower/yellow
    pub const TRACK_GREEN: Self = Self(61, 220, 132);     // #3DDC84 green flag
    pub const TRACK_YELLOW: Self = Self(245, 197, 66);    // #F5C542 yellow flag
    pub const TRACK_SC: Self = Self(255, 159, 28);        // #FF9F1C safety car / VSC
    pub const TRACK_RED: Self = Self(225, 6, 0);          // #E10600 red flag
    pub const CHEQUERRED: Self = Self(0, 0, 0);          // black/white chequered
}

/// Tyre compound badge colours (truecolor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TyreColour(pub u8, pub u8, pub u8);

impl TyreColour {
    pub const SOFT: Self = Self(255, 51, 51);    // #FF3333 red
    pub const MEDIUM: Self = Self(255, 209, 46);  // #FFD12E yellow
    pub const HARD: Self = Self(240, 240, 240);  // #F0F0F0 white
    pub const INTERMEDIATE: Self = Self(57, 181, 74); // #39B54A green
    pub const WET: Self = Self(45, 140, 255);    // #2D8CFF blue
    pub const NEUTRAL: Self = Self(170, 170, 170); // #AAAAAA fallback

    pub fn from_compound(compound: &str) -> Self {
        match compound.to_lowercase().as_str() {
            "soft" | "S" => Self::SOFT,
            "medium" | "M" => Self::MEDIUM,
            "hard" | "H" => Self::HARD,
            "intermediate" | "I" => Self::INTERMEDIATE,
            "wet" | "W" => Self::WET,
            _ => Self::NEUTRAL,
        }
    }
}

/// Colours used in the timing tower sector bars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SectorColour {
    Purple,
    Green,
    Yellow,
}

/// Return the team colour for a driver code, falling back to neutral.
pub fn team_colour_for_code(code: &str) -> TeamColour {
    TeamColour::from_code(code)
}

/// Return the tyre compound colour.
pub fn tyre_colour(compound: &str) -> TyreColour {
    TyreColour::from_compound(compound)
}

/// Return the UI colour for a given role.
pub fn ui_colour(role: &str) -> UiColour {
    match role {
        "border_unfocused" => UiColour::BORDER_UNFOCUSED,
        "border_focused" => UiColour::BORDER_FOCUSED,
        "primary_text" => UiColour::PRIMARY_TEXT,
        "dim_stale" => UiColour::DIM_STALE,
        "purple_sector" => UiColour::PURPLE_SECTOR,
        "personal_best" => UiColour::PERSONAL_BEST,
        "slower" => UiColour::SLOWER,
        "track_green" => UiColour::TRACK_GREEN,
        "track_yellow" => UiColour::TRACK_YELLOW,
        "track_sc" => UiColour::TRACK_SC,
        "track_red" => UiColour::TRACK_RED,
        "chequered" => UiColour::CHEQUERRED,
        _ => UiColour::PRIMARY_TEXT,
    }
}

/// Colour-blind theme swaps: red/green deltas become blue/orange.
/// Called when `config.units` or `config.colour_blind` is set.
pub fn apply_colour_blind(style: &ratatui::style::Style) -> ratatui::style::Style {
    let fg = style.fg;
    let bg = style.bg;

    let (r, g, b) = match fg {
        Some(ratatui::style::Color::Rgb(r, g, b)) => (*r, *g, *b),
        None => (0, 0, 0),
    };

    // Determine the dominant colour channel
    let is_red_dominant = r > g && r > b;
    let is_green_dominant = g > r && g > b;

    // Build new colours: swap red-dominant to orange, green-dominant to orange too
    let new_fg = if is_red_dominant || is_green_dominant {
        Some(ratatui::style::Color::Rgb(255, 165, 0)) // orange
    } else {
        fg
    };

    let new_bg = bg; // keep background unchanged

    ratatui::style::Style::new().fg(new_fg).bg(new_bg)
}

/// Check if the terminal supports truecolor.
pub fn supports_truecolor() -> bool {
    std::env::var("COLORTERM")
        .map(|v| v == "truecolor" || v == "24bit")
        .unwrap_or(false)
}

/// Map truecolor to nearest ANSI-256 colour index (0-255).
/// Simple palette reduction for terminals without truecolor support.
pub fn ansi256_from_rgb(r: u8, g: u8, b: u8) -> u8 {
    // Fast approximation: use the nearest of the 6x6x6 cube + 24 gray + 24 black/white
    let r_idx = (r as f32 / 255.0 * 5.0).round() as u8;
    let g_idx = (g as f32 / 255.0 * 5.0).round() as u8;
    let b_idx = (b as f32 / 255.0 * 5.0).round() as u8;
    let cube = 16 + (r_idx * 36) + (g_idx * 6) + b_idx;

    // If result would be > 231, clamp to 232-255 grayscale range
    if cube > 231 {
        let gray = 232 + ((r + g + b) as f32 / 765.0 * 24.0).round() as u8;
        gray
    } else {
        cube
    }
}