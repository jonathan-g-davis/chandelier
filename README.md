# chandelier

[![crates.io](https://img.shields.io/crates/v/chandelier.svg)](https://crates.io/crates/chandelier)
[![Documentation](https://docs.rs/chandelier/badge.svg)](https://docs.rs/chandelier)
[![CI](https://github.com/jonathan-g-davis/chandelier/actions/workflows/ci.yml/badge.svg)](https://github.com/jonathan-g-davis/chandelier/actions/workflows/ci.yml)
[![MIT/Apache 2.0 licensed](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Financial charting widgets for [Ratatui](https://ratatui.rs).

## Usage

A chart is a `CandleSeries` (the bars and how they are colored) drawn by a
`CandlestickChart` (the container that autoscales the price axis, lays out the
plot, and draws the axes). The chart implements `Widget`, so render it like any
other Ratatui widget:

```rust
use chandelier::{Candle, CandleSeries, CandlestickChart, PriceAxis, TimeAxis};
use ratatui::style::{Color, Style};

// OHLC bars come from your own data source.
let candles = [
    Candle::new(100.0, 105.0, 99.0, 104.0),
    Candle::new(104.0, 108.0, 103.0, 106.0),
    Candle::new(106.0, 107.0, 101.0, 102.0),
];

let series = CandleSeries::new(&candles)
    .bull_style(Color::Rgb(38, 166, 154))
    .bear_style(Color::Rgb(239, 83, 80))
    .wick_style(Color::Rgb(110, 116, 130));

let chart = CandlestickChart::new(series)
    .style(Style::new().bg(Color::Rgb(13, 17, 23)))
    .price_axis(PriceAxis::default())
    .time_axis(TimeAxis::default());

// In your draw closure:
// frame.render_widget(&chart, area);
```

See [`examples/candlestick.rs`](examples/candlestick.rs) for a complete runnable
terminal app (`cargo run --example candlestick`).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
