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
    let columns = cores.len().div_ceil(rows).max(1);
    let column_width = (width as usize).saturating_sub(columns - 1) / columns.max(1);
    let compact = column_width < 15;
    let used_rows = cores.len().min(rows);

    if column_width < 8 {
        return vec![Line::from(Span::styled(
            "expand terminal width",
            Style::default().fg(Color::DarkGray),
        ))];
    }
    let mut lines = Vec::with_capacity(used_rows);

    for row in 0..used_rows {
        let mut spans = Vec::new();
        for col in 0..columns {
            let idx = row + col * rows;
            if idx >= cores.len() {
                continue;
            }
            if col > 0 {
                spans.push(Span::raw(" "));
            }
            spans.extend(core_bar(idx, cores[idx], column_width, compact, theme));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn core_bar(
    index: usize,
    value: f32,
    width: usize,
    compact: bool,
    theme: ThemeName,
) -> Vec<Span<'static>> {
    if compact {
        let cell = format!("{index:02}:{value:>3.0}%");
        return vec![Span::styled(
            pad_to(&cell, width),
            Style::default().fg(load_color(theme, value)),
        )];
    }

    let label = format!("{index:02} ");
    let pct = format!(" {:>4.0}%", value);
    let bar_width = width.saturating_sub(label.len() + pct.len()).max(4);
    let filled = ((value / 100.0) * bar_width as f32).round() as usize;

    let mut spans = Vec::with_capacity(4);
    spans.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(
        "▌".repeat(filled.min(bar_width)),
        Style::default().fg(load_color(theme, value)),
    ));
    spans.push(Span::styled(
        " ".repeat(bar_width.saturating_sub(filled)),
        Style::default()
            .fg(Color::Rgb(24, 27, 33))
            .bg(Color::Rgb(24, 27, 33)),
    ));
    spans.push(Span::styled(
        pct,
        Style::default().fg(load_color(theme, value)),
    ));
    spans
}

fn pad_to(content: &str, width: usize) -> String {
    if content.len() >= width {
        content[..width].to_string()
    } else {
        let mut out = String::with_capacity(width);
        out.push_str(content);
        out.push_str(&" ".repeat(width - content.len()));
        out
    }
}
