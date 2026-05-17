# rtop

A fast Rust terminal system monitor inspired by btop, built with Ratatui.

## Current Features

- CPU and memory panels
- High-resolution Braille graphs
- Per-core CPU box that adapts layout to show all cores
- Centered CPU graph that grows above and below the baseline
- Configurable sampling interval (default 200ms)
- Theme support: `ocean`, `ember`, `mono`
- Bounded history buffers for stable memory usage
- Non-blocking sampler channel to avoid stale metric buildup

## Run

```sh
cargo run --release
```

## Configuration

CLI options:

```sh
cargo run --release -- --interval 150 --theme ember
```

Environment variables:

```sh
RTOP_INTERVAL_MS=150 RTOP_THEME=mono cargo run --release
```

Runtime keybindings:

- `q` / `Esc`: quit
- `+` / `=`: increase sampling interval by 25ms
- `-`: decrease sampling interval by 25ms
- `t`: cycle theme (`ocean` -> `ember` -> `mono`)

Press `q` or `Esc` to quit.

## Status

Early prototype. The first goal is a smooth, lightweight monitor with great-looking terminal graphs.
