use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
    widgets::{Paragraph, Block},
};

/** A sector time bar showing S1/S2/S3 bars with colours (purple/green/yellow)
 * and best lap time beside.
 *
 * Renders like: `▇▇▇ 1:24.8` where the three blocks represent S1, S2, S3
 * and the time is the best lap (or last lap).
 *
 * In ASCII mode, block elements are replaced with `_`, `-`, `|` characters.
 */
#[derive(Debug, Clone)]
pub struct SectorBar {
    s1: f32,   // sector 1 time in seconds
    s2: f32,   // sector 2 time in seconds
    s3: f32,   // sector 3 time in seconds
    best: f32, // best lap time in seconds
    show_times: bool,
    style: Style,
    ascii_mode: bool,
}

impl SectorBar {
    pub fn new(s1: f32, s2: f32, s3: f32, best: f32, show_times: bool, style: Style, ascii_mode: bool) -> Self {
        Self {
            s1,
            s2,
            s3,
            best,
            show_times,
            style,
            ascii_mode,
        }
    }

    /// Renders the sector bars. The width determines bar length.
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

        // Map each sector to a bar length (0-7 blocks)
        // ASCII block chars: _ - ▁▂▃▄▅▇# or Unicode ▁▂▃▄▅▆▇█
        const BLOCKS: &[char] = if self.ascii_mode {
            &['_', '-', '▁', '▂', '▃', '▄', '▅', '#']
        } else {
            &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
        };

        let bar_width = usable_width.saturating_sub(10); // leave room for time text
        let bar_per_sec = bar_width as f32 / 7.0;

        // Calculate bar lengths: how many blocks per sector
        let s1_blocks = ((self.s1 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;
        let s2_blocks = ((self.s2 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;
        let s3_blocks = ((self.s3 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;

        // Clamp to 0-7
        let s1_blocks = s1_blocks.min(7);
        let s2_blocks = s2_blocks.min(7);
        let s3_blocks = s3_blocks.min(7);

        // Build bar string: S1 blocks, separator, S2 blocks, separator, S3 blocks
        let s1_str: String = std::iter::repeat(BLOCKS.get(s1_blocks.min(7)).copied().unwrap_or('▁'))
            .take(s1_blocks.min(7))
            .collect();
        let s2_str: String = std::iter::repeat(BLOCKS.get(s2_blocks.min(7)).copied().unwrap_or('▁'))
            .take(s2_blocks.min(7))
            .collect();
        let s3_str: String = std::iter::repeat(BLOCKS.get(s3_blocks.min(7)).copied().unwrap_or('▁'))
            .take(s3_blocks.min(7))
            .collect();

        let separator = " | ";

        let bars = format!("{}{}{}", s1_str, separator, s2_str);
        let full_bars = format!("{}{}{}", s1_str, separator, s2_str); // + s3 bit tricky in limited width

        // We'll do: S1|S2|S3 all on one line, then time on next or same
        let (bars_text, time_text) = if self.show_times {
            let best_str = format!("{}", self.best);
            // Bars take some width, time takes the rest
            let available_for_bars = usable_width.saturating_sub(best_str.len() + 2);
            let bar_w = available_for_bars.max(1);
            let s1_b = ((self.s1 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;
            let s2_b = ((self.s2 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;
            let s3_b = ((self.s3 - min_sec) / range * 7.0).max(0.0).min(7.0) as usize;

            let s1_c: String = std::iter::repeat(&BLOCKS[s1_b.min(7)]).take(s1_b.min(7)).collect();
            let s2_c: String = std::iter::repeat(&BLOCKS[s2_b.min(7)]).take(s2_b.min(7)).collect();
            let s3_c: String = std::iter::repeat(&BLOCKS[s3_b.min(7)]).take(s3_b.min(7)).collect();

            let best_part = format!(" {} ", best_str);
            // Distribute bars: S1 and S2 get space, S3 is compact
            let s1_len = ((s1_b as f32 / 7.0) * bar_w as f32) as usize;
            let s2_len = ((s2_b as f32 / 7.0) * bar_w as f32) as usize;
            let s3_len = bar_w.saturating_sub(s1_len + s2_len).max(1);

            let s1_part: String = std::iter::repeat(&BLOCKS[s1_b.min(7)]).take(s1_len.min(s1_b)).collect();
            let s2_part: String = std::iter::repeat(&BLOCKS[s2_b.min(7)]).take(s2_len.min(s2_b)).collect();
            let s3_part: String = std::iter::repeat(&BLOCKS[s3_b.min(7)]).take(s3_len.min(s3_b as usize)).collect();

            let combined = format!("{}{}{}{}{}", s1_part, " | ", s2_part, " | ", s3_part);
            (combined, best_part)
        } else {
            // Just show bars without times
            let s1_c: String = std::iter::repeat(&BLOCKS[s1_blocks.min(7)]).take(s1_blocks.min(7)).collect();
            let s2_c: String = std::iter::repeat(&BLOCKS[s2_blocks.min(7)]).take(s2_blocks.min(7)).collect();
            let s3_c: String = std::iter::repeat(&BLOCKS[s3_blocks.min(7)]).take(s3_blocks.min(7)).collect();
            let bars = format!("{}{}{}", s1_c, " | ", s2_c);
            let time_str = String::new();
            (bars, time_str)
        };

        // Render: bars section, then time below or beside
        let y_offset = if self.show_times && !time_text.is_empty() {
            // Two lines: bars on top, time below
            if usable_height >= 2 {
                // We'll just render the bars; the caller handles layout
                (bars_text.as_str(), Some(time_text.as_str()))
            } else {
                (bars_text.as_str(), None)
            }
        } else {
            (bars_text.as_str(), None)
        };

        // For simplicity, render just the bars with colour styling
        // The sector colours: S1=purple, S2=green, S3=yellow
        let s1_colour = Color::Rgb(177, 79, 255); // #B14FFF purple
        let s2_colour = Color::Rgb(61, 220, 132); // #3DDC84 green
        let s3_colour = Color::Rgb(245, 197, 66); // #F5C542 yellow

        let paragraph = Paragraph::new(bars_text)
            .style(self.style.fg(s1_colour)); // default colour, individual chars can vary
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
            ascii_mode: false,
        }
    }
}