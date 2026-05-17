use std::collections::VecDeque;

use crate::metrics::Metrics;

pub const HISTORY_LEN: usize = 420;

#[derive(Debug)]
pub struct App {
    pub cpu_history: VecDeque<f32>,
    pub memory_history: VecDeque<f32>,
    pub current: Metrics,
}

impl App {
    pub fn new(initial: Metrics) -> Self {
        let mut app = Self {
            cpu_history: VecDeque::with_capacity(HISTORY_LEN),
            memory_history: VecDeque::with_capacity(HISTORY_LEN),
            current: initial,
        };
        app.push(app.current.clone());
        app
    }

    pub fn push(&mut self, metrics: Metrics) {
        push_bounded(&mut self.cpu_history, metrics.cpu);
        push_bounded(&mut self.memory_history, metrics.memory);
        self.current = metrics;
    }
}

fn push_bounded(history: &mut VecDeque<f32>, value: f32) {
    if history.len() == HISTORY_LEN {
        history.pop_front();
    }
    history.push_back(value);
}
