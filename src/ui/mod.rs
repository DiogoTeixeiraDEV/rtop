mod color;
mod cores;
mod graph;
mod processes;

use std::collections::VecDeque;

use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::App,
    config::{AppConfig, ThemeName},
};

use self::{
    color::{Palette, bytes_to_gib, load_color},
    cores::core_lines,
    graph::{braille_graph, braille_graph_top_down},
    processes::process_lines,
};

pub fn render(frame: &mut ratatui::Frame<'_>, app: &App, config: &AppConfig) {
    let area = frame.size();
    frame.render_widget(
        Block::default().style(Style::default().bg(theme_background(config.theme))),
        area,
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_cpu_panel(
        frame,
        chunks[0],
        app.current.cpu,
        app.current.cpu_temp_c,
        &app.current.cores,
        &app.cpu_history,
        config,
    );
    render_memory_and_processes(
        frame,
        chunks[1],
        app.current.memory,
        app.current.used_memory,
        app.current.total_memory,
        &app.memory_history,
        &app.current.processes,
        config,
    );
}

fn render_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    title: Line<'static>,
    history: &VecDeque<f32>,
    palette: Palette,
    config: &AppConfig,
) {
    let border = neutral_border();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 || inner.width < 8 {
        return;
    }

    let graph = braille_graph(
        history,
        inner.width as usize,
        inner.height as usize,
        palette,
        config.theme,
    );
    frame.render_widget(Paragraph::new(graph), inner);
}

fn render_memory_and_processes(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    memory_value: f32,
    used_memory: u64,
    total_memory: u64,
    history: &VecDeque<f32>,
    processes: &[crate::metrics::ProcessInfo],
    config: &AppConfig,
) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let title = Line::from(vec![
        Span::styled(" Memory ", Style::default().fg(Color::White)),
        Span::styled(
            format!(
                "{memory_value:>5.1}%   {:.1}/{:.1} GiB ",
                bytes_to_gib(used_memory),
                bytes_to_gib(total_memory)
            ),
            Style::default().fg(Color::Rgb(205, 208, 214)),
        ),
    ]);
    render_panel(frame, split[0], title, history, Palette::Memory, config);

    render_process_panel(frame, split[1], processes, config);
}

fn render_process_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    processes: &[crate::metrics::ProcessInfo],
    config: &AppConfig,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(vec![Span::styled(
            " Processes ",
            Style::default().fg(Color::White),
        )]))
        .border_style(Style::default().fg(neutral_border()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height < 1 || inner.width < 12 {
        return;
    }
    frame.render_widget(
        Paragraph::new(process_lines(
            processes,
            inner.width,
            inner.height,
            config.theme,
        )),
        inner,
    );
}

fn render_cpu_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    value: f32,
    cpu_temp_c: Option<f32>,
    cores: &[f32],
    history: &VecDeque<f32>,
    config: &AppConfig,
) {
    let palette = Palette::Cpu;
    let border = neutral_border();
    let title_left = Line::from(vec![
        Span::styled(" CPU ", Style::default().fg(Color::White)),
    ]);
    let title_right = Line::from(vec![
        Span::styled(" + ", Style::default().fg(load_color(config.theme, 65.0))),
        Span::styled(
            format!("{}ms", config.sample_interval.as_millis()),
            Style::default().fg(Color::Rgb(205, 208, 214)),
        ),
        Span::styled(" - ", Style::default().fg(load_color(config.theme, 65.0))),
    ])
    .alignment(Alignment::Right);
    let title_center = Line::from(vec![Span::styled(
        format!(" {} ", Local::now().format("%H:%M:%S")),
        Style::default().fg(Color::Rgb(205, 208, 214)),
    )])
    .alignment(Alignment::Center);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_left)
        .title(title_center)
        .title(title_right)
        .border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 6 || inner.width < 24 {
        return;
    }

    let core_box_width = inner.width.clamp(22, 44);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(core_box_width)])
        .split(inner);
    let left = columns[0];
    let right = columns[1];

    let graph_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(left);
    let top_graph = braille_graph(
        history,
        graph_split[0].width as usize,
        graph_split[0].height as usize,
        palette,
        config.theme,
    );
    frame.render_widget(Paragraph::new(top_graph), graph_split[0]);
    let bottom_graph = braille_graph_top_down(
        history,
        graph_split[2].width as usize,
        graph_split[2].height as usize,
        palette,
        config.theme,
    );
    frame.render_widget(Paragraph::new(bottom_graph), graph_split[2]);

    let center_line_area = graph_split[1];
    let temp = cpu_temp_c
        .map(|t| format!("{t:.0}°C"))
        .unwrap_or_else(|| "N/A".to_string());
    let center_label = format!(" {}  {:.1}% ", temp, value);
    frame.render_widget(
        Paragraph::new(make_center_rule(center_line_area.width as usize, &center_label))
            .style(Style::default().fg(Color::Rgb(168, 156, 133))),
        center_line_area,
    );

    let desired_height = if right.height >= 12 {
        ((right.height as f32) * 0.72) as u16
    } else if right.height >= 8 {
        right.height.saturating_sub(2)
    } else {
        right.height
    }
    .max(5)
    .min(right.height);
    let top_pad = right.height.saturating_sub(desired_height) / 2;
    let right_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(desired_height),
            Constraint::Min(0),
        ])
        .split(right);
    let core_area = right_v[1];

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(vec![Span::styled(
                " Cores ",
                Style::default().fg(Color::White),
            )]))
            .border_style(Style::default().fg(border)),
        core_area,
    );
    let core_inner = Block::default().borders(Borders::ALL).inner(core_area);
    frame.render_widget(
        Paragraph::new(core_lines(
            cores,
            core_inner.width,
            core_inner.height,
            config.theme,
        )),
        core_inner,
    );
}

fn neutral_border() -> Color {
    Color::Rgb(128, 132, 138)
}

fn make_center_rule(width: usize, label: &str) -> String {
    if width == 0 {
        return String::new();
    }
    let label = if label.len() + 2 > width {
        &label[..width.saturating_sub(2)]
    } else {
        label
    };
    let side = width.saturating_sub(label.len()) / 2;
    let mut out = String::with_capacity(width);
    out.push_str(&"─".repeat(side));
    out.push_str(label);
    out.push_str(&"─".repeat(width.saturating_sub(side + label.len())));
    out
}

fn theme_background(theme: ThemeName) -> Color {
    match theme {
        ThemeName::GruvboxDark => Color::Rgb(20, 18, 16),
        ThemeName::Ocean => Color::Rgb(10, 14, 20),
        ThemeName::Ember => Color::Rgb(18, 12, 10),
        ThemeName::Mono => Color::Rgb(10, 10, 10),
    }
}
