mod color;
mod cores;
mod graph;
mod processes;

use std::collections::VecDeque;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::{app::App, config::AppConfig};

use self::{
    color::{Palette, bytes_to_gib},
    cores::core_lines,
    graph::{braille_graph, braille_graph_centered},
    processes::process_lines,
};

pub fn render(frame: &mut ratatui::Frame<'_>, app: &App, config: &AppConfig) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    render_header(frame, chunks[0], app, config);
    render_cpu_panel(
        frame,
        chunks[1],
        app.current.cpu,
        &app.current.cores,
        &app.cpu_history,
        config,
    );
    render_memory_and_processes(
        frame,
        chunks[2],
        app.current.memory,
        &app.memory_history,
        &app.current.processes,
        config,
    );
}

fn render_header(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App, config: &AppConfig) {
    let used = bytes_to_gib(app.current.used_memory);
    let total = bytes_to_gib(app.current.total_memory);
    let age_ms = app.current.sampled_at.elapsed().as_millis();
    let line = Line::from(vec![
        Span::styled(
            "rtop",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::raw("q/esc quit  +/- interval  t theme"),
        Span::raw("  "),
        Span::styled(
            format!("{}ms samples", config.sample_interval.as_millis()),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("mem {used:.1}/{total:.1} GiB"),
            Style::default().fg(Color::LightGreen),
        ),
        Span::raw("  "),
        Span::styled(
            format!("theme {}", config.theme.as_str()),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("age {age_ms}ms"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        area,
    );
}

fn render_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    title: &'static str,
    value: f32,
    history: &VecDeque<f32>,
    palette: Palette,
    config: &AppConfig,
) {
    let color = palette.accent(config.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} {:>5.1}% ", value))
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 || inner.width < 8 {
        return;
    }

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let graph = braille_graph(
        history,
        split[0].width as usize,
        split[0].height as usize,
        palette,
        config.theme,
    );
    frame.render_widget(Paragraph::new(graph), split[0]);

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::Black))
            .ratio((value / 100.0).clamp(0.0, 1.0) as f64)
            .label(format!("{value:.1}%")),
        split[1],
    );
}

fn render_memory_and_processes(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    memory_value: f32,
    history: &VecDeque<f32>,
    processes: &[crate::metrics::ProcessInfo],
    config: &AppConfig,
) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_panel(
        frame,
        split[0],
        "Memory",
        memory_value,
        history,
        Palette::Memory,
        config,
    );

    render_process_panel(frame, split[1], processes, config);
}

fn render_process_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    processes: &[crate::metrics::ProcessInfo],
    config: &AppConfig,
) {
    let border = match config.theme {
        crate::config::ThemeName::Ocean => Color::Rgb(98, 132, 181),
        crate::config::ThemeName::Ember => Color::Rgb(186, 115, 88),
        crate::config::ThemeName::Mono => Color::Rgb(140, 140, 140),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Processes by RAM ")
        .border_style(Style::default().fg(border));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height < 1 || inner.width < 12 {
        return;
    }
    frame.render_widget(
        Paragraph::new(process_lines(processes, inner.width, inner.height)),
        inner,
    );
}

fn render_cpu_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    value: f32,
    cores: &[f32],
    history: &VecDeque<f32>,
    config: &AppConfig,
) {
    let palette = Palette::Cpu;
    let color = palette.accent(config.theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" CPU {:>5.1}% ", value))
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 6 || inner.width < 24 {
        return;
    }

    let core_box_width = inner.width.clamp(20, 42);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(core_box_width)])
        .split(inner);
    let left = columns[0];
    let right = columns[1];

    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(2), Constraint::Length(1)])
        .split(left);

    let graph = braille_graph_centered(
        history,
        left_split[0].width as usize,
        left_split[0].height as usize,
        palette,
        config.theme,
    );
    frame.render_widget(Paragraph::new(graph), left_split[0]);

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::Black))
            .ratio((value / 100.0).clamp(0.0, 1.0) as f64)
            .label(format!("{value:.1}%")),
        left_split[1],
    );

    let core_box_height = right.height;
    let top_pad = right.height.saturating_sub(core_box_height) / 2;
    let right_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(core_box_height),
            Constraint::Min(0),
        ])
        .split(right);
    let core_area = right_v[1];

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Cores ")
            .border_style(Style::default().fg(Color::Rgb(90, 110, 140))),
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
