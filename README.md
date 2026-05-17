# rtop

A fast Rust terminal system monitor inspired by btop, built with Ratatui.

## Current Features

- CPU and memory panels
- High-resolution Braille graphs
- Per-core CPU usage bars
- 200ms metrics sampling
- Bounded history buffers for stable memory usage
- Non-blocking sampler channel to avoid stale metric buildup

## Run

```sh
cargo run --release
```

Press `q` or `Esc` to quit.

## Status

Early prototype. The first goal is a smooth, lightweight monitor with great-looking terminal graphs.
