/// Sparkline widget using block elements `▁▂▃▄▅▆▇█` (8 levels).
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
    widgets::Paragraph,
};

/// A sparkline widget using block elements.
#[derive(Debug, Clone)]
pub struct Sparkline {
    data: Vec<f32>,
    fixed_min: Option<f32>,
    fixed_max: Option<f32>,
    style: Style,
}

impl Sparkline {
    pub fn new(data: &[f32], style: Style) -> Self {
        Self {
            data: data.to_vec(),
            fixed_min: None,
            fixed_max: None,
            style,
        }
    }

    pub fn with_range(data: &[f32], min: f32, max: f32, style: Style) -> Self {
        Self {
            data: data.to_vec(),
            fixed_min: Some(min),
            fixed_max: Some(max),
            style,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let usable_height = area.height.max(1);
        let usable_width = area.width.max(1);

        if self.data.is_empty() || usable_width < 2 || usable_height < 2 {
            return;
        }

        // Determine min/max for scaling
        let (min, max) = if let (Some(fixed_min), Some(fixed_max)) = (self.fixed_min, self.fixed_max) {
            let clamped: Vec<f32> = self.data.iter().map(|v| v.max(fixed_min).min(fixed_max)).collect();
            (fixed_min, fixed_max)
        } else {
            let min_val = self.data.iter().cloned().fold(f32::INFINITY, |a, b| a.min(b));
            let max_val = self.data.iter().cloned().fold(f32::NEG_INFINITY, |a, b| a.max(b));
            let range = if max_val - min_val == 0.0 { 1.0 } else { max_val - min_val };
            (min_val, max_val)
        };

        // Block element levels: ▁▂▃▄▅▆▇█
        const LEVELS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        let range_val = (max - min).max(1.0);

        // Render as a single line of block elements
        let mut output = String::with_capacity(usable_width);
        for &val in self.data.iter().take(usable_width) {
            let norm = ((val - min) / range_val).max(0.0).min(1.0);
            let level = (norm * 7.0).floor() as usize;
            output.push(LEVELS[level.min(7)]);
        }

        // Pad or truncate to fit usable_width
        if output.len() < usable_width {
            output.push_str(&" ".repeat(usable_width - output.len()));
        } else {
            output.truncate(usable_width);
        }

        let paragraph = ratatui::widgets::Paragraph::new(output)
            .style(self.style)
            .alignment(ratatui::layout::Alignment::Right);
        f.render_widget(paragraph, area);
    }
}

impl Default for Sparkline {
    fn default() -> Self {
        Self {
            data: vec![0.0],
            fixed_min: None,
            fixed_max: None,
            style: ratatui::style::Style::default(),
        }
    }
}