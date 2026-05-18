use std::{env, time::Duration};

pub const DEFAULT_SAMPLE_MS: u64 = 200;
pub const DEFAULT_FRAME_MS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Ocean,
    Ember,
    Mono,
    GruvboxDark,
}

impl ThemeName {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ocean" => Some(Self::Ocean),
            "ember" => Some(Self::Ember),
            "mono" | "monochrome" => Some(Self::Mono),
            "gruvbox" | "gruvbox-dark" | "gruvbox_dark" => Some(Self::GruvboxDark),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Ocean => Self::Ember,
            Self::Ember => Self::Mono,
            Self::Mono => Self::GruvboxDark,
            Self::GruvboxDark => Self::Ocean,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub sample_interval: Duration,
    pub frame_interval: Duration,
    pub theme: ThemeName,
}

impl AppConfig {
    pub fn load() -> Self {
        let mut sample_ms = DEFAULT_SAMPLE_MS;
        let frame_ms = DEFAULT_FRAME_MS;
        let mut theme = ThemeName::GruvboxDark;

        if let Ok(value) = env::var("RTOP_INTERVAL_MS") {
            if let Ok(parsed) = value.parse::<u64>() {
                sample_ms = parsed.clamp(50, 5000);
            }
        }
        if let Ok(value) = env::var("RTOP_THEME") {
            if let Some(parsed) = ThemeName::parse(&value) {
                theme = parsed;
            }
        }

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--interval" => {
                    if let Some(value) = args.next() {
                        if let Ok(parsed) = value.parse::<u64>() {
                            sample_ms = parsed.clamp(50, 5000);
                        }
                    }
                }
                "--theme" => {
                    if let Some(value) = args.next() {
                        if let Some(parsed) = ThemeName::parse(&value) {
                            theme = parsed;
                        }
                    }
                }
                _ => {}
            }
        }

        Self {
            sample_interval: Duration::from_millis(sample_ms),
            frame_interval: Duration::from_millis(frame_ms),
            theme,
        }
    }
}
