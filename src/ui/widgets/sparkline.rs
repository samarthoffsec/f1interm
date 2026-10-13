use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
};

/**
 * Sparkline widget using block elements `▁▂▃▄▅▆▇█` (8 levels).
 *
 * Renders a compact sparkline sparkline for a sequence of f32 values.
 * The Y-axis is auto-scaled to the min/max of the data, or clamped if
 * fixed_min/fixed_max are Some.
 */
#[derive(Debug, Clone)]
pub struct Sparkline {
    data: Vec<f32>,
    fixed_min: Option<f32>,
    fixed_max: Option<f32>,
    style: Style,
}

impl Sparkline {
    pub fn new(data: &[f32], style: Style) -> Self {
        let fixed_min = None;
        let fixed_max = None;
        Self {
            data: data.to_vec(),
            fixed_min,
            fixed_max,
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
            // Clamp data to fixed range
            let clamped: Vec<f32> = self.data.iter().map(|v| v.max(fixed_min).min(fixed_max)).collect();
            (fixed_min, fixed_max)
        } else {
            let min_val = self.data.iter().cloned().fold(f32::INFINITY, |a, b| a.min(b));
            let max_val = self.data.iter().cloned().fold(f32::NEG_INFINITY, |a, b| a.max(b));
            // Avoid zero range
            let range = if max_val - min_val == 0.0 {
                1.0
            } else {
                max_val - min_val
            };
            (min_val, max_val)
        };

        // Block element levels: ▁▂▃▄▅▆▇█ (0-7)
        const LEVELS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        let range = (max - min).max(1.0);

        // Render as a single line of block elements
        let mut output = String::with_capacity(usable_width);
        for (i, &val) in self.data.iter().enumerate() {
            // Clamp value to range and map to level 0-7
            let norm = ((val - min) / range).max(0.0).min(1.0);
            let level = (norm * 7.0).floor() as usize;
            let idx = LEVELS.get(level).copied().unwrap_or(' ');
            // Approximate cell width: each char is roughly 2 terminal columns
            // But we just output one char per data point and rely on terminal aspect
            output.push(idx);
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

/// A simple gauge using block elements, useful for throttle (0-100%) or brake.
#[derive(Debug, Clone)]
pub struct Gauge {
    value: f32,      // 0.0 - 1.0
    label: String,
    style: Style,
    min: f32,
    max: f32,
}

impl Gauge {
    pub fn new(value: f32, label: &str, style: Style) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            label: label.to_string(),
            style,
            min: 0.0,
            max: 1.0,
        }
    }

    pub fn with_range(value: f32, min: f32, max: f32, label: &str, style: Style) -> Self {
        Self {
            value: value.clamp(min, max),
            label: label.to_string(),
            style,
            min,
            max,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let usable_width = area.width.max(1);
        let usable_height = area.height.max(1);

        if usable_width < 3 || usable_height < 3 {
            return;
        }

        // Determine filled portion based on clamped value
        let val = self.value.clamp(self.min, self.max);
        let range = (self.max - self.min).max(1.0);
        let filled = ((val - self.min) / range).max(0.0).min(1.0);
        let num_blocks = (filled * 7.0).floor() as usize; // 0-7 blocks

        // Block elements ▁▂▃▄▅▆▇█
        const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        // We'll render: label on left, bars in middle, value on right
        let bar_section = usable_width.saturating_sub(self.label.len() + 8);
        let bar_len = bar_section.max(1);

        let mut bar_str = String::with_capacity(bar_len);
        for i in 0..bar_len {
            if i < num_blocks {
                bar_str.push(BLOCKS[i.min(7)]);
            } else {
                bar_str.push('░'); // empty block
            }
        }

        // Pad or truncate
        let bar_str = if bar_str.len() >= bar_len {
            bar_str.truncate(bar_len);
            bar_str
        } else {
            bar_str & pad_to(&bar_str, bar_len, ' ')
        };

        let text = format!("{} {} {}", self.label, bar_str, self.value);
        let paragraph = ratatui::widgets::Paragraph::new(text)
            .style(self.style)
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(paragraph, area);
    }
}

fn pad_to(s: &str, target: usize, pad_char: char) -> String {
    let mut result = s.to_string();
    result.push_str(&pad_char.repeat(target - s.len()));
    result
}

impl Default for Gauge {
    fn default() -> Self {
        Self {
            value: 0.0,
            label: String::new(),
            style: ratatui::style::Style::default(),
            min: 0.0,
            max: 1.0,
        }
    }
}