/// Sector time bar showing S1/S2/S3 bars with colours (purple/green/yellow)
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
    widgets::{Paragraph, Block},
};

/// A sector time bar showing S1/S2/S3 bars with colours and best lap time.
#[derive(Debug, Clone)]
pub struct SectorBar {
    s1: f32,
    s2: f32,
    s3: f32,
    best: f32,
    show_times: bool,
    style: Style,
}

impl SectorBar {
    pub fn new(s1: f32, s2: f32, s3: f32, best: f32, show_times: bool, style: Style) -> Self {
        Self {
            s1,
            s2,
            s3,
            best,
            show_times,
            style,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let usable_width = area.width.max(1);
        let usable_height = area.height.max(1);

        if usable_width < 5 {
            return;
        }

        // Determine max sector time for scaling
        let max_sec = [self.s1, self.s2, self.s3]
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, |a, b| a.max(b));
        let min_sec = [self.s1, self.s2, self.s3]
            .iter()
            .cloned()
            .fold(f32::INFINITY, |a, b| a.min(b));

        let range = if max_sec - min_sec == 0.0 {
            1.0
        } else {
            max_sec - min_sec
        };

        // Block elements ▁▂▃▄▅▆▇█
        const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        // Map each sector to a bar length (0-7 blocks)
        let s1_blocks = ((self.s1 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;
        let s2_blocks = ((self.s2 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;
        let s3_blocks = ((self.s3 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;

        // Clamp to 0-7
        let s1_blocks = s1_blocks.min(7);
        let s2_blocks = s2_blocks.min(7);
        let s3_blocks = s3_blocks.min(7);

        // Build bar strings
        let s1_str: String = std::iter::repeat(BLOCKS[s1_blocks]).take(s1_blocks).collect();
        let s2_str: String = std::iter::repeat(BLOCKS[s2_blocks]).take(s2_blocks).collect();
        let s3_str: String = std::iter::repeat(BLOCKS[s3_blocks]).take(s3_blocks).collect();

        let separator = " | ";

        let (bars_text, time_text) = if self.show_times {
            let best_str = format!("{}", self.best);
            let available_for_bars = usable_width.saturating_sub(best_str.len() + 2);
            let bar_w = available_for_bars.max(1);
            let s1_len = ((s1_blocks as f32 / 7.0) * bar_w as f32) as usize;
            let s2_len = ((s2_blocks as f32 / 7.0) * bar_w as f32) as usize;
            let s3_len = bar_w.saturating_sub(s1_len + s2_len).max(1);

            let s1_part: String = std::iter::repeat(&BLOCKS[s1_blocks]).take(s1_len.min(s1_blocks)).collect();
            let s2_part: String = std::iter::repeat(&BLOCKS[s2_blocks]).take(s2_len.min(s2_blocks)).collect();
            let s3_part: String = std::iter::repeat(&BLOCKS[s3_blocks]).take(s3_len.min(s3_blocks)).collect();

            let combined = format!("{}{}{}{}{}", s1_part, " | ", s2_part, " | ", s3_part);
            (combined, best_part)
        } else {
            let s1_c: String = std::iter::repeat(&BLOCKS[s1_blocks]).take(s1_blocks).collect();
            let s2_c: String = std::iter::repeat(&BLOCKS[s2_blocks]).take(s2_blocks).collect();
            let s3_c: String = std::iter::repeat(&BLOCKS[s3_blocks]).take(s3_blocks).collect();
            let bars = format!("{}{}{}", s1_c, " | ", s2_c);
            let time_str = String::new();
            (bars, time_str)
        };

        // Render: bars section
        let s1_colour = Color::Rgb(177, 79, 255); // purple
        let s2_colour = Color::Rgb(61, 220, 132); // green
        let s3_colour = Color::Rgb(245, 197, 66); // yellow

        let paragraph = Paragraph::new(bars_text)
            .style(self.style.fg(s1_colour));
        f.render_widget(paragraph, area);
    }
}

impl Default for SectorBar {
    fn default() -> Self {
        Self {
            s1: 0.0,
            s2: 0.0,
            s3: 0.0,
            best: 0.0,
            show_times: true,
            style: ratatui::style::Style::default(),
        }
    }
}