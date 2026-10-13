use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
};

/**
 * A custom Braille graph widget using 2×4 (8-level) sub-cell resolution.
 *
 * Each Braille cell covers 2 columns × 4 rows (8 vertical dot positions).
 * We map f32 values to one of 8 heights, rendering across the available width.
 *
 * When `ascii_mode` is enabled, Braille characters are replaced with
 * ASCII alternatives (`#`, `+`, `-`, `|`) for terminals without Braille support.
 */
#[derive(Debug, Clone)]
pub struct BrailleGraph {
    /// Normalized data values in [0, 1] range
    data: Vec<f32>,
    /// Maximum visual height in Braille rows (typically 3-4 per widget area)
    max_height: usize,
    /// Whether to show a border around the graph area
    show_border: bool,
    /// Whether to render using ASCII characters instead of Braille
    ascii_mode: bool,
}

impl BrailleGraph {
    pub fn new(data: &[f32], max_height: usize, show_border: bool, ascii_mode: bool) -> Self {
        let data_norm = if data.is_empty() {
            vec![0.0]
        } else {
            let min = data.iter().cloned().fold(f32::INFINITY, |a, b| a.min(b));
            let max = data.iter().cloned().fold(f32::NEG_INFINITY, |a, b| a.max(b));
            if max == min {
                vec![0.0; data.len()]
            } else {
                data.iter()
                    .map(|v| (v - min) / (max - min))
                    .collect()
            }
        };
        Self {
            data: data_norm,
            max_height,
            show_border,
            ascii_mode,
        }
    }

    /// Render the graph into the given area using the provided style.
    pub fn render(&self, f: &mut Frame, area: Rect, style: Style) {
        let height = area.height.saturating_sub(if self.show_border { 2 } else { 0 });
        let usable_height = height.max(1);
        let usable_width = area.width.max(1);

        if self.data.is_empty() || usable_height < 2 || usable_width < 2 {
            return;
        }

        // Calculate step per value cell
        let cell_width = 2; // 2 columns per braille char
        let cell_height_rows = usable_height;

        // Prepare character sets
        let (braille_chars, ascii_chars) = if self.ascii_mode {
            // ASCII fallback: use block elements for visual density
            // Level 0: space, Level 1: |, Level 2: -, Level 3: |, Level 4: -, Level 5: #, Level 6: #, Level 7: full block
            (vec![' ', '|', '-', '|', '-', '#', '#', '█'], vec![' ', '|', '-', '|', '-', '#', '#', '█'])
        } else {
            // Braille patterns for 8 vertical levels (2x4 dot grid)
            // Bit 0 = bottom dot
            const BRAILLE: [char; 8] = ['⠀', '⠁', '⠃', '⠉', '⠋', '⠪', '⠫', '⠻'];
            (BRAILLE, BRAILLE)
        };

        // We'll render row by row from top to bottom
        let mut output_lines: Vec<String> = Vec::new();

        for row_idx in 0..=cell_height_rows {
            let mut line = String::new();
            let mut chars_rendered = 0usize;

            let mut char_chars: Vec<char> = Vec::new();

            // Process data in pairs (each braille char takes 2 columns)
            for chunk in self.data.chunks(2) {
                let mut val_sum: f32 = 0.0;
                let mut count = 0usize;
                for &v in chunk.iter() {
                    val_sum += v;
                    count += 1;
                }
                let avg = if count > 0 { val_sum / count as f32 } else { 0.0 };

                // Map [0,1] to braille level 0-7
                let level = (avg * 7.0).min(7.0).max(0.0) as usize;
                char_chars.push(braille_chars[level]);
                chars_rendered += cell_width;
            }

            // If we have remaining space, fill with empty chars
            while chars_rendered < usable_width {
                char_chars.push(if self.ascii_mode { ' ' } else { '⠀' });
                chars_rendered += cell_width;
            }

            // Truncate to fit area width
            let n_chars = usable_width / cell_width;
            let line_str: String = char_chars.into_iter().take(n_chars).collect();
            output_lines.push(line_str);
        }

        // Reverse so row 0 is top (we built bottom-up)
        let output_lines: Vec<String> = output_lines.into_iter().rev().collect();

        // Draw using Paragraph
        use ratatui::widgets::Paragraph;
        let paragraph = Paragraph::new(output_lines.join("\n"))
            .style(style)
            .block(if self.show_border {
                ratatui::widgets::Block::bordered()
            } else {
                ratatui::widgets::Block::default()
            });
        f.render_widget(paragraph, area);
    }
}

impl Default for BrailleGraph {
    fn default() -> Self {
        Self {
            data: vec![0.0],
            max_height: 4,
            show_border: true,
            ascii_mode: false,
        }
    }
}

impl Default for BrailleGraph {
    fn default() -> Self {
        Self {
            data: vec![0.0],
            max_height: 4,
            show_border: true,
        }
    }
}