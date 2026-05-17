use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::metrics::ProcessInfo;

pub fn process_lines(processes: &[ProcessInfo], width: u16, height: u16) -> Vec<Line<'static>> {
    if height == 0 || width < 14 {
        return vec![];
    }

    let w = width as usize;
    let pid_w = 6_usize.min(w.saturating_sub(8)).max(3);
    let ram_w = 9_usize.min(w.saturating_sub(pid_w + 3)).max(6);
    let name_w = w.saturating_sub(pid_w + ram_w + 2);

    let mut lines = Vec::with_capacity(height as usize);
    lines.push(Line::from(vec![
        Span::styled(pad("PID", pid_w), Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(pad("Name", name_w), Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::styled(pad_left("RAM", ram_w), Style::default().fg(Color::DarkGray)),
    ]));

    let max_rows = height.saturating_sub(1) as usize;
    for process in processes.iter().take(max_rows) {
        lines.push(Line::from(vec![
            Span::styled(
                pad_left(&process.pid.to_string(), pid_w),
                Style::default().fg(Color::Rgb(145, 150, 165)),
            ),
            Span::raw(" "),
            Span::styled(
                pad(&trim_name(&process.name, name_w), name_w),
                Style::default().fg(Color::Rgb(220, 225, 235)),
            ),
            Span::raw(" "),
            Span::styled(
                pad_left(&format_mem(process.memory_bytes), ram_w),
                Style::default().fg(Color::Rgb(110, 210, 185)),
            ),
        ]));
    }

    lines
}

fn format_mem(bytes: u64) -> String {
    let mib = bytes as f64 / 1024.0 / 1024.0;
    if mib < 1024.0 {
        format!("{mib:.0} MiB")
    } else {
        format!("{:.1} GiB", mib / 1024.0)
    }
}

fn trim_name(name: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    for ch in name.chars().take(width) {
        out.push(ch);
    }
    out
}

fn pad(text: &str, width: usize) -> String {
    if text.len() >= width {
        text[..width].to_string()
    } else {
        format!("{text:<width$}")
    }
}

fn pad_left(text: &str, width: usize) -> String {
    if text.len() >= width {
        text[text.len() - width..].to_string()
    } else {
        format!("{text:>width$}")
    }
}
