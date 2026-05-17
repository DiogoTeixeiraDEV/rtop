use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub cpu: f32,
    pub cores: Vec<f32>,
    pub memory: f32,
    pub used_memory: u64,
    pub total_memory: u64,
    pub sampled_at: Instant,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum SamplerCommand {
    SetInterval(Duration),
}

pub struct SamplerHandle {
    pub rx: Receiver<Metrics>,
    pub tx: Sender<SamplerCommand>,
}

pub fn default_metrics() -> Metrics {
    Metrics {
        cpu: 0.0,
        cores: Vec::new(),
        memory: 0.0,
        used_memory: 0,
        total_memory: 0,
        sampled_at: Instant::now(),
        processes: Vec::new(),
    }
}

pub fn spawn_sampler(initial_interval: Duration) -> SamplerHandle {
    let (tx, rx) = mpsc::sync_channel(2);
    let (cmd_tx, cmd_rx) = mpsc::channel::<SamplerCommand>();

    thread::spawn(move || {
        let refresh = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(ProcessRefreshKind::everything());
        let mut system = System::new_with_specifics(refresh);
        let mut interval = initial_interval;

        loop {
            while let Ok(command) = cmd_rx.try_recv() {
                match command {
                    SamplerCommand::SetInterval(new_interval) => interval = new_interval,
                }
            }

            let start = Instant::now();
            system.refresh_cpu();
            system.refresh_memory();
            system.refresh_processes();

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
            let mut processes: Vec<ProcessInfo> = system
                .processes()
                .iter()
                .map(|(pid, process)| ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    memory_bytes: process.memory(),
                })
                .collect();
            processes.sort_by(|a, b| {
                b.memory_bytes
                    .cmp(&a.memory_bytes)
                    .then_with(|| a.name.cmp(&b.name))
            });
            processes.truncate(128);

            let _ = tx.try_send(Metrics {
                cpu,
                cores,
                memory,
                used_memory,
                total_memory,
                sampled_at: Instant::now(),
                processes,
            });

            let elapsed = start.elapsed();
            if elapsed < interval {
                thread::sleep(interval - elapsed);
            }
        }
    });

    SamplerHandle { rx, tx: cmd_tx }
}
