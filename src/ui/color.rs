use ratatui::style::Color;

use crate::config::ThemeName;

#[derive(Clone, Copy)]
pub enum Palette {
    Cpu,
    Memory,
}

impl Palette {
    pub fn color(self, theme: ThemeName, value: f32, age: f32) -> Color {
        let base = match (theme, self) {
            (ThemeName::Ocean, Self::Cpu) => gradient(
                value,
                &[
                    (0.0, (28, 210, 230)),
                    (45.0, (70, 230, 160)),
                    (75.0, (245, 210, 85)),
                    (100.0, (255, 95, 105)),
                ],
            ),
            (ThemeName::Ocean, Self::Memory) => gradient(
                value,
                &[
                    (0.0, (95, 220, 155)),
                    (55.0, (110, 220, 235)),
                    (80.0, (245, 205, 90)),
                    (100.0, (255, 110, 95)),
                ],
            ),
            (ThemeName::Ember, Self::Cpu) => gradient(
                value,
                &[
                    (0.0, (255, 170, 110)),
                    (40.0, (255, 126, 95)),
                    (75.0, (245, 84, 84)),
                    (100.0, (186, 49, 49)),
                ],
            ),
            (ThemeName::Ember, Self::Memory) => gradient(
                value,
                &[
                    (0.0, (250, 206, 97)),
                    (50.0, (255, 151, 83)),
                    (80.0, (255, 101, 101)),
                    (100.0, (180, 64, 64)),
                ],
            ),
            (ThemeName::Mono, Self::Cpu) | (ThemeName::Mono, Self::Memory) => gradient(
                value,
                &[
                    (0.0, (110, 110, 110)),
                    (50.0, (170, 170, 170)),
                    (80.0, (220, 220, 220)),
                    (100.0, (255, 255, 255)),
                ],
            ),
            (ThemeName::GruvboxDark, Self::Cpu) => gradient(
                value,
                &[
                    (0.0, (184, 187, 38)),
                    (45.0, (250, 189, 47)),
                    (78.0, (254, 128, 25)),
                    (100.0, (204, 36, 29)),
                ],
            ),
            (ThemeName::GruvboxDark, Self::Memory) => gradient(
                value,
                &[
                    (0.0, (131, 165, 152)),
                    (35.0, (142, 192, 124)),
                    (65.0, (250, 189, 47)),
                    (85.0, (254, 128, 25)),
                    (100.0, (251, 73, 52)),
                ],
            ),
        };
        let _ = age;
        Color::Rgb(base.0, base.1, base.2)
    }
}

pub fn load_color(theme: ThemeName, value: f32) -> Color {
    let (r, g, b) = match theme {
        ThemeName::Ocean => gradient(
            value,
            &[
                (0.0, (62, 220, 235)),
                (55.0, (82, 235, 145)),
                (80.0, (245, 205, 85)),
                (100.0, (255, 90, 100)),
            ],
        ),
        ThemeName::Ember => gradient(
            value,
            &[
                (0.0, (255, 194, 106)),
                (50.0, (255, 132, 92)),
                (80.0, (241, 83, 83)),
                (100.0, (176, 52, 52)),
            ],
        ),
        ThemeName::Mono => gradient(
            value,
            &[
                (0.0, (120, 120, 120)),
                (50.0, (180, 180, 180)),
                (80.0, (225, 225, 225)),
                (100.0, (255, 255, 255)),
            ],
        ),
        ThemeName::GruvboxDark => gradient(
            value,
            &[
                (0.0, (184, 187, 38)),
                (40.0, (250, 189, 47)),
                (70.0, (254, 128, 25)),
                (100.0, (251, 73, 52)),
            ],
        ),
    };
    Color::Rgb(r, g, b)
}

pub fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
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
