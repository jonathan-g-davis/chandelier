# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/jonathan-g-davis/chandelier/compare/v0.1.0...v0.2.0) - 2026-06-02

### Other

- Add axis label alignment ([#11](https://github.com/jonathan-g-davis/chandelier/pull/11))
- Add block wrapping to chart ([#10](https://github.com/jonathan-g-davis/chandelier/pull/10))
- Move marker to CandleSeries ([#9](https://github.com/jonathan-g-davis/chandelier/pull/9))
- Render refactor ([#8](https://github.com/jonathan-g-davis/chandelier/pull/8))
- Add box drawing backend ([#7](https://github.com/jonathan-g-davis/chandelier/pull/7))
- Add quadrants backend ([#6](https://github.com/jonathan-g-davis/chandelier/pull/6))
- Add fill styles ([#5](https://github.com/jonathan-g-davis/chandelier/pull/5))
- Braille rendering backend ([#3](https://github.com/jonathan-g-davis/chandelier/pull/3))

## [0.1.0](https://github.com/jonathan-g-davis/chandelier/releases/tag/v0.1.0) - 2026-05-31

Initial release. A candlestick chart widget for Ratatui.

### Added

- `CandlestickChart`, a Ratatui `Widget` that autoscales the price axis to the
  data in view and draws the most recent candles that fit, right aligned. It
  composes a `CandleSeries` with a `PriceAxis` and a `TimeAxis`.
- `CandleSeries`, the candle dataset: a slice of open/high/low/close `Candle`
  bars plus per-series bull, bear, and wick styles and configurable candle width
  and gap.
- `PriceAxis` and `TimeAxis`, which style the right-hand price axis and the
  bottom time axis independently. The price axis selects round-numbered ticks
  automatically; the time axis shows candle indices or caller-supplied labels.
- Sub-cell rendering. Body open and close endpoints resolve to an eighth of a
  row and wick tips to a half row, using partial block and vertical box-drawing
  glyphs. Partial bodies render correctly over any background, including the
  terminal default, so a chart background is optional.
- Styling through `Style`, with `Styled` implemented on the chart and both axes
  so `Stylize` shorthands work. The chart's base style background, when set, is
  the color the chart area is filled with.
- `PriceScale` and `TimeScale`, the invertible price-to-row and index-to-column
  coordinate maps the renderer is built on.
- `price_bounds`, a helper for the lowest low and highest high across candles.
- A runnable terminal example (`cargo run --example candlestick`).
