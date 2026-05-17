mod app;
mod config;
mod metrics;
mod terminal;
mod ui;

use std::{io, thread, time::Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::App,
    config::AppConfig,
    metrics::{SamplerCommand, default_metrics, spawn_sampler},
    terminal::{restore_terminal, setup_terminal},
    ui::render,
};

fn main() -> io::Result<()> {
    let config = AppConfig::load();
    let sampler = spawn_sampler(config.sample_interval);
    let first = sampler.rx.recv().unwrap_or_else(|_| default_metrics());

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, sampler, App::new(first), config);
    restore_terminal(&mut terminal)?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    sampler: metrics::SamplerHandle,
    mut app: App,
    mut config: AppConfig,
) -> io::Result<()> {
    let mut next_frame = Instant::now();

    loop {
        while let Ok(metrics) = sampler.rx.try_recv() {
            app.push(metrics);
        }

        if event::poll(std::time::Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('t') => {
                            config.theme = config.theme.next();
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            let ms =
                                (config.sample_interval.as_millis() as u64 + 25).clamp(50, 5000);
                            config.sample_interval = std::time::Duration::from_millis(ms);
                            let _ = sampler
                                .tx
                                .send(SamplerCommand::SetInterval(config.sample_interval));
                        }
                        KeyCode::Char('-') => {
                            let current = config.sample_interval.as_millis() as u64;
                            let ms = current.saturating_sub(25).clamp(50, 5000);
                            config.sample_interval = std::time::Duration::from_millis(ms);
                            let _ = sampler
                                .tx
                                .send(SamplerCommand::SetInterval(config.sample_interval));
                        }
                        _ => {}
                    }
                }
            }
        }

        if Instant::now() >= next_frame {
            terminal.draw(|frame| render(frame, &app, &config))?;
            next_frame = Instant::now() + config.frame_interval;
        }

        thread::sleep(std::time::Duration::from_millis(8));
    }
}
