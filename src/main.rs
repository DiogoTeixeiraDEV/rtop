use std::{
    collections::VecDeque,
    io::{self, Stdout},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(200);
const FRAME_INTERVAL: Duration = Duration::from_millis(100);
const HISTORY_LEN: usize = 420;

#[derive(Debug, Clone)]
struct Metrics {
    cpu: f32,
    cores: Vec<f32>,
    memory: f32,
    used_memory: u64,
    total_memory: u64,
    sampled_at: Instant,
}

#[derive(Debug)]
struct App {
    cpu_history: VecDeque<f32>,
    memory_history: VecDeque<f32>,
    current: Metrics,
}

impl App {
    fn new(initial: Metrics) -> Self {
        let mut app = Self {
            cpu_history: VecDeque::with_capacity(HISTORY_LEN),
            memory_history: VecDeque::with_capacity(HISTORY_LEN),
            current: initial,
        };
        app.push(app.current.clone());
        app
    }

    fn push(&mut self, metrics: Metrics) {
        push_bounded(&mut self.cpu_history, metrics.cpu);
        push_bounded(&mut self.memory_history, metrics.memory);
        self.current = metrics;
    }
}

fn main() -> io::Result<()> {
    let rx = spawn_sampler(SAMPLE_INTERVAL);
    let first = rx.recv().unwrap_or_else(|_| Metrics {
        cpu: 0.0,
        cores: Vec::new(),
        memory: 0.0,
        used_memory: 0,
        total_memory: 0,
        sampled_at: Instant::now(),
    });

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, rx, App::new(first));
    restore_terminal(&mut terminal)?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    rx: Receiver<Metrics>,
    mut app: App,
) -> io::Result<()> {
    let mut next_frame = Instant::now();

    loop {
        while let Ok(metrics) = rx.try_recv() {
            app.push(metrics);
        }

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    return Ok(());
                }
            }
        }

        if Instant::now() >= next_frame {
            terminal.draw(|frame| render(frame, &app))?;
            next_frame = Instant::now() + FRAME_INTERVAL;
        }

        thread::sleep(Duration::from_millis(8));
    }
}

fn render(frame: &mut ratatui::Frame<'_>, app: &App) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    render_header(frame, chunks[0], app);
    render_cpu_panel(
        frame,
        chunks[1],
        app.current.cpu,
        &app.current.cores,
        &app.cpu_history,
    );
    render_panel(
        frame,
        chunks[2],
        "Memory",
        app.current.memory,
        &app.memory_history,
        Palette::Memory,
    );
}

fn render_header(frame: &mut ratatui::Frame<'_>, area: Rect, app: &App) {
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
        Span::raw("q/esc quit"),
        Span::raw("  "),
        Span::styled(
            format!("{}ms samples", SAMPLE_INTERVAL.as_millis()),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            format!("mem {used:.1}/{total:.1} GiB"),
            Style::default().fg(Color::LightGreen),
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
) {
    let color = palette.accent();
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

fn render_cpu_panel(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    value: f32,
    cores: &[f32],
    history: &VecDeque<f32>,
) {
    let palette = Palette::Cpu;
    let color = palette.accent();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" CPU {:>5.1}% ", value))
        .border_style(Style::default().fg(color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 5 || inner.width < 16 {
        return;
    }

    let core_rows = core_bar_rows(cores.len()).min(inner.height.saturating_sub(3));
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),
            Constraint::Length(core_rows),
            Constraint::Length(1),
        ])
        .split(inner);

    let graph = braille_graph(
        history,
        split[0].width as usize,
        split[0].height as usize,
        palette,
    );
    frame.render_widget(Paragraph::new(graph), split[0]);

    frame.render_widget(Paragraph::new(core_lines(cores, split[1].width)), split[1]);

    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::Black))
            .ratio((value / 100.0).clamp(0.0, 1.0) as f64)
            .label(format!("{value:.1}%")),
        split[2],
    );
}

fn spawn_sampler(interval: Duration) -> Receiver<Metrics> {
    let (tx, rx) = mpsc::sync_channel(2);

    thread::spawn(move || {
        let refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        let mut system = System::new_with_specifics(refresh);

        loop {
            let start = Instant::now();
            system.refresh_cpu();
            system.refresh_memory();

            let cpu = system.global_cpu_info().cpu_usage().clamp(0.0, 100.0);
            let cores = system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage().clamp(0.0, 100.0))
                .collect();
            let total_memory = system.total_memory();
            let used_memory = system.used_memory();
            let memory = if total_memory == 0 {
                0.0
            } else {
                (used_memory as f32 / total_memory as f32 * 100.0).clamp(0.0, 100.0)
            };

            let _ = tx.try_send(Metrics {
                cpu,
                cores,
                memory,
                used_memory,
                total_memory,
                sampled_at: Instant::now(),
            });

            let elapsed = start.elapsed();
            if elapsed < interval {
                thread::sleep(interval - elapsed);
            }
        }
    });

    rx
}

fn braille_graph(
    history: &VecDeque<f32>,
    width: usize,
    height: usize,
    palette: Palette,
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
                palette.color(peak, age)
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

fn core_bar_rows(core_count: usize) -> u16 {
    if core_count == 0 {
        1
    } else {
        core_count.div_ceil(2) as u16
    }
}

fn core_lines(cores: &[f32], width: u16) -> Vec<Line<'static>> {
    if cores.is_empty() {
        return vec![Line::from(Span::styled(
            "waiting for cores...",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let columns = if width >= 56 { 2 } else { 1 };
    let column_width = (width as usize).saturating_sub(columns - 1) / columns;
    let rows = cores.len().div_ceil(columns);
    let mut lines = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut spans = Vec::new();
        for col in 0..columns {
            let idx = row + col * rows;
            if idx >= cores.len() {
                continue;
            }
            if col > 0 {
                spans.push(Span::raw(" "));
            }
            spans.extend(core_bar(idx, cores[idx], column_width));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn core_bar(index: usize, value: f32, width: usize) -> Vec<Span<'static>> {
    let label = format!("{index:02} ");
    let pct = format!(" {:>4.0}%", value);
    let bar_width = width.saturating_sub(label.len() + pct.len()).max(4);
    let filled = ((value / 100.0) * bar_width as f32).round() as usize;

    let mut spans = Vec::with_capacity(4);
    spans.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(
        "▌".repeat(filled.min(bar_width)),
        Style::default().fg(load_color(value)),
    ));
    spans.push(Span::styled(
        " ".repeat(bar_width.saturating_sub(filled)),
        Style::default()
            .fg(Color::Rgb(24, 27, 33))
            .bg(Color::Rgb(24, 27, 33)),
    ));
    spans.push(Span::styled(pct, Style::default().fg(load_color(value))));
    spans
}

#[derive(Clone, Copy)]
enum Palette {
    Cpu,
    Memory,
}

impl Palette {
    fn accent(self) -> Color {
        match self {
            Self::Cpu => Color::LightCyan,
            Self::Memory => Color::LightGreen,
        }
    }

    fn color(self, value: f32, age: f32) -> Color {
        let base = match self {
            Self::Cpu => gradient(
                value,
                &[
                    (0.0, (28, 210, 230)),
                    (45.0, (70, 230, 160)),
                    (75.0, (245, 210, 85)),
                    (100.0, (255, 95, 105)),
                ],
            ),
            Self::Memory => gradient(
                value,
                &[
                    (0.0, (95, 220, 155)),
                    (55.0, (110, 220, 235)),
                    (80.0, (245, 205, 90)),
                    (100.0, (255, 110, 95)),
                ],
            ),
        };
        let fade = 0.45 + age.clamp(0.0, 1.0) * 0.55;
        Color::Rgb(
            (base.0 as f32 * fade) as u8,
            (base.1 as f32 * fade) as u8,
            (base.2 as f32 * fade) as u8,
        )
    }
}

fn load_color(value: f32) -> Color {
    let (r, g, b) = gradient(
        value,
        &[
            (0.0, (62, 220, 235)),
            (55.0, (82, 235, 145)),
            (80.0, (245, 205, 85)),
            (100.0, (255, 90, 100)),
        ],
    );
    Color::Rgb(r, g, b)
}

fn gradient(value: f32, stops: &[(f32, (u8, u8, u8))]) -> (u8, u8, u8) {
    let value = value.clamp(0.0, 100.0);
    for pair in stops.windows(2) {
        let (lo_value, lo_color) = pair[0];
        let (hi_value, hi_color) = pair[1];
        if value <= hi_value {
            let t = ((value - lo_value) / (hi_value - lo_value)).clamp(0.0, 1.0);
            return lerp_rgb(lo_color, hi_color, t);
        }
    }
    stops
        .last()
        .map(|(_, color)| *color)
        .unwrap_or((255, 255, 255))
}

fn lerp_rgb(from: (u8, u8, u8), to: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (
        lerp_channel(from.0, to.0, t),
        lerp_channel(from.1, to.1, t),
        lerp_channel(from.2, to.2, t),
    )
}

fn lerp_channel(from: u8, to: u8, t: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * t) as u8
}

fn push_bounded(history: &mut VecDeque<f32>, value: f32) {
    if history.len() == HISTORY_LEN {
        history.pop_front();
    }
    history.push_back(value);
}

fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}
