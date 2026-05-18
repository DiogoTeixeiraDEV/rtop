use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use sysinfo::{
    Components, CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System,
};

const PROCESS_REFRESH_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone)]
pub struct Metrics {
    pub cpu: f32,
    pub cores: Vec<f32>,
    pub cpu_temp_c: Option<f32>,
    pub memory: f32,
    pub used_memory: u64,
    pub total_memory: u64,
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
        cpu_temp_c: None,
        memory: 0.0,
        used_memory: 0,
        total_memory: 0,
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
        let mut components = Components::new_with_refreshed_list();
        let mut interval = initial_interval;
        let mut process_refresh_at = Instant::now();
        let mut cached_processes: Vec<ProcessInfo> = Vec::with_capacity(128);

        loop {
            while let Ok(command) = cmd_rx.try_recv() {
                match command {
                    SamplerCommand::SetInterval(new_interval) => interval = new_interval,
                }
            }

            let start = Instant::now();
            system.refresh_cpu();
            system.refresh_memory();
            components.refresh();
            if Instant::now() >= process_refresh_at {
                system.refresh_processes();
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
                cached_processes = processes;
                process_refresh_at = Instant::now() + PROCESS_REFRESH_INTERVAL;
            }

            let cpu = system.global_cpu_info().cpu_usage().clamp(0.0, 100.0);
            let cores = system
                .cpus()
                .iter()
                .map(|cpu| cpu.cpu_usage().clamp(0.0, 100.0))
                .collect();
            let cpu_temp_c = pick_cpu_temperature(&components);
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
                cpu_temp_c,
                memory,
                used_memory,
                total_memory,
                processes: cached_processes.clone(),
            });

            let elapsed = start.elapsed();
            if elapsed < interval {
                thread::sleep(interval - elapsed);
            }
        }
    });

    SamplerHandle { rx, tx: cmd_tx }
}

fn pick_cpu_temperature(components: &Components) -> Option<f32> {
    let mut cpu_specific: Option<f32> = None;
    let mut max_temp: Option<f32> = None;

    for component in components.list() {
        let temp = component.temperature();
        if !temp.is_finite() || temp <= 0.0 {
            continue;
        }
        let label = component.label().to_ascii_lowercase();
        if label.contains("cpu") || label.contains("package") {
            cpu_specific = Some(cpu_specific.map_or(temp, |current| current.max(temp)));
        }
        max_temp = Some(max_temp.map_or(temp, |current| current.max(temp)));
    }

    cpu_specific.or(max_temp)
}
