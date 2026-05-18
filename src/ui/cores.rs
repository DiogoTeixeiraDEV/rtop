use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{config::ThemeName, ui::color::load_color};

pub fn core_lines(cores: &[f32], width: u16, height: u16, theme: ThemeName) -> Vec<Line<'static>> {
    if cores.is_empty() {
        return vec![Line::from(Span::styled(
            "waiting for cores...",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let rows = height.max(1) as usize;
    let data_rows = rows.div_ceil(2).max(1);
    let columns = cores.len().div_ceil(data_rows).max(1);
    let column_width = (width as usize).saturating_sub(columns - 1) / columns.max(1);
    let used_rows = cores.len().min(data_rows);

    if column_width < 8 {
        return vec![Line::from(Span::styled(
            "expand terminal width",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let mut lines = Vec::with_capacity(rows);
    for row in 0..used_rows {
        let mut spans = Vec::new();
        for col in 0..columns {
            let idx = row + col * data_rows;
            if idx >= cores.len() {
                continue;
            }
            if col > 0 {
                spans.push(Span::raw(" "));
            }
            spans.extend(core_braille_row(idx, cores[idx], column_width, theme));
        }
        lines.push(Line::from(spans));
        if lines.len() < rows && row + 1 < used_rows {
            lines.push(Line::from(Span::styled(
                "·".repeat(width as usize),
                Style::default().fg(Color::Rgb(48, 51, 58)),
            )));
        }
    }

    lines
}

fn core_braille_row(
    index: usize,
    value: f32,
    width: usize,
    theme: ThemeName,
) -> Vec<Span<'static>> {
    let label = format!("{index:02}");
    let pct = format!("{value:>3.0}%");
    let bar_width = width.saturating_sub(label.len() + pct.len() + 2).max(3);
    let bar = braille_bar(value, bar_width);

    vec![
        Span::styled(label, Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(bar, Style::default().fg(load_color(theme, value))),
        Span::raw(" "),
        Span::styled(pct, Style::default().fg(load_color(theme, value))),
    ]
}

fn braille_bar(value: f32, width: usize) -> String {
    let mut out = String::with_capacity(width);
    let level = (value.clamp(0.0, 100.0) / 100.0) * width as f32;
    for i in 0..width {
        let local = (level - i as f32).clamp(0.0, 1.0);
        out.push(quantized_braille(local));
    }
    out
}

fn quantized_braille(v: f32) -> char {
    match (v * 8.0).round() as u8 {
        0 => '⠀',
        1 => '⢀',
        2 => '⢠',
        3 => '⢰',
        4 => '⢸',
        5 => '⣸',
        6 => '⣾',
        7 => '⣿',
        _ => '⣿',
    }
}
