/// Braille graph widget for line graphs using block elements.
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
    widgets::Paragraph,
};

/// A simple line graph widget using block elements.
#[derive(Debug, Clone)]
pub struct BrailleGraph {
    data: Vec<f32>,
    max_height: usize,
    show_border: bool,
}

impl BrailleGraph {
    pub fn new(data: &[f32], max_height: usize, show_border: bool) -> Self {
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
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, style: Style) {
        let height = area.height.saturating_sub(if self.show_border { 2 } else { 0 });
        let usable_height = height.max(1);
        let usable_width = area.width.max(1);

        if self.data.is_empty() || usable_height < 2 || usable_width < 2 {
            return;
        }

        // Block element levels: ▁▂▃▄▅▆▇█
        const LEVELS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        let mut output_lines: Vec<String> = Vec::new();

        for row_idx in 0..=usable_height {
            let mut line = String::new();
            for chunk in self.data.chunks(2) {
                let avg: f32 = chunk.iter().sum::<f32>() / chunk.len() as f32;
                let level = ((avg * 7.0).clamp(0.0, 1.0) * 7.0) as usize;
                line.push(LEVELS[level]);
            }
            output_lines.push(line);
        }

        let output_lines: Vec<String> = output_lines.into_iter().rev().collect();

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
        }
    }
}