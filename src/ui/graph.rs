use std::collections::VecDeque;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::config::ThemeName;
use crate::ui::color::Palette;

pub fn braille_graph(
    history: &VecDeque<f32>,
    width: usize,
    height: usize,
    palette: Palette,
    theme: ThemeName,
) -> Vec<Line<'static>> {
    let sample_width = width.saturating_mul(2);
    let graph_height = height.saturating_mul(4);
    if sample_width == 0 || graph_height == 0 {
        return Vec::new();
    }

    let mut values = vec![0.0; sample_width];
    let visible = history.len().min(sample_width);
    let start = history.len().saturating_sub(visible);
    let left_pad = sample_width - visible;

    for (idx, value) in history.iter().skip(start).enumerate() {
        values[left_pad + idx] = *value;
    }

    let mut lines = Vec::with_capacity(height);
    for row in 0..height {
        let mut spans = Vec::with_capacity(width);
        for col in 0..width {
            let mut bits = 0u8;
            let mut peak = 0.0_f32;
            for x in 0..2 {
                let value = values[col * 2 + x];
                peak = peak.max(value);
                let filled = ((value / 100.0) * graph_height as f32).round() as usize;

                for y in 0..4 {
                    let from_bottom = graph_height - (row * 4 + y);
                    if filled >= from_bottom {
                        bits |= braille_bit(x, y);
                    }
                }
            }
            let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
            let age = col as f32 / width.max(1) as f32;
            let fg = if bits == 0 {
                Color::Rgb(35, 39, 47)
            } else {
                palette.color(theme, peak, age)
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(fg)));
        }
        lines.push(Line::from(spans));
    }

    lines
}

pub fn braille_graph_centered(
    history: &VecDeque<f32>,
    width: usize,
    height: usize,
    palette: Palette,
    theme: ThemeName,
) -> Vec<Line<'static>> {
    let sample_width = width.saturating_mul(2);
    let center = height / 2;
    let top_rows = center;
    let bottom_rows = height.saturating_sub(center);
    let amp_rows = top_rows.min(bottom_rows).max(1);
    let amp_dots = amp_rows.saturating_mul(4);
    if sample_width == 0 || amp_dots == 0 {
        return Vec::new();
    }

    let mut values = vec![0.0; sample_width];
    let visible = history.len().min(sample_width);
    let start = history.len().saturating_sub(visible);
    let left_pad = sample_width - visible;
    for (idx, value) in history.iter().skip(start).enumerate() {
        values[left_pad + idx] = *value;
    }

    let mut lines = Vec::with_capacity(height);
    for row in 0..height {
        let mut spans = Vec::with_capacity(width);
        for col in 0..width {
            let mut bits = 0u8;
            let mut peak = 0.0_f32;
            for x in 0..2 {
                let value = values[col * 2 + x];
                peak = peak.max(value);
                let amp = ((value / 100.0) * amp_dots as f32).round() as usize;
                for y in 0..4 {
                    let dot_row = row * 4 + y;
                    let center_dot = center * 4;
                    let dist = dot_row.abs_diff(center_dot);
                    if dist <= amp {
                        bits |= braille_bit(x, y);
                    }
                }
            }
            let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
            let age = col as f32 / width.max(1) as f32;
            let fg = if bits == 0 {
                if row == center {
                    Color::Rgb(55, 62, 73)
                } else {
                    Color::Rgb(28, 31, 37)
                }
            } else {
                palette.color(theme, peak, age)
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(fg)));
        }
        lines.push(Line::from(spans));
    }
    lines
}

fn braille_bit(x: usize, y: usize) -> u8 {
    match (x, y) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (0, 3) => 0x40,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (1, 3) => 0x80,
        _ => 0,
    }
}
