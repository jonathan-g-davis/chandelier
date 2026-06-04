# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/jonathan-g-davis/chandelier/compare/v0.3.2...v0.4.0) - 2026-06-04

### Other

- Support defined axis ticks and bounds ([#23](https://github.com/jonathan-g-davis/chandelier/pull/23))
- Add Line charts ([#21](https://github.com/jonathan-g-davis/chandelier/pull/21))

## [0.3.2](https://github.com/jonathan-g-davis/chandelier/compare/v0.3.1...v0.3.2) - 2026-06-04

### Added

- Line chart overlays for plotting on a candlestick or volume chart, such as an SMA ([#18](https://github.com/jonathan-g-davis/chandelier/pull/18))
- Support underlays to allow drawing behind the candlesticks/bars ([#20](https://github.com/jonathan-g-davis/chandelier/pull/20))

## [0.3.1](https://github.com/jonathan-g-davis/chandelier/compare/v0.3.0...v0.3.1) - 2026-06-04

### Added

- Horizontal trend line overlays for marking support, resistance, last price, etc. ([#15](https://github.com/jonathan-g-davis/chandelier/pull/15))
- Symbol+label annotations on charts, such as for BUY/SELL markers ([#17](https://github.com/jonathan-g-davis/chandelier/pull/17))

### Fixed

- Centering logic for aligning labels, markers to candle bodies

## [0.3.0](https://github.com/jonathan-g-davis/chandelier/compare/v0.2.0...v0.3.0) - 2026-06-03

### Added

- Add volume charts for rendering trade volume data ([#14](https://github.com/jonathan-g-davis/chandelier/pull/14))

### Removed

- Dropped `PlotLayout`, `PriceScale`, and `TimeScale` from the public API.

### Other

- Add README badges ([#12](https://github.com/jonathan-g-davis/chandelier/pull/12))

## [0.2.0](https://github.com/jonathan-g-davis/chandelier/compare/v0.1.0...v0.2.0) - 2026-06-02

### Added

- Braille rendering backend ([#3](https://github.com/jonathan-g-davis/chandelier/pull/3))
- Block quadrants backend ([#6](https://github.com/jonathan-g-davis/chandelier/pull/6))
- Box drawing backend ([#7](https://github.com/jonathan-g-davis/chandelier/pull/7))
- Fill styles for rendering candle bodies as filled or hollow ([#5](https://github.com/jonathan-g-davis/chandelier/pull/5))
- `CandlestickChart` can be wrapped in `Block` ([#10](https://github.com/jonathan-g-davis/chandelier/pull/10))
- Label alignment on `PriceAxis` and `TimeAxis` ([#11](https://github.com/jonathan-g-davis/chandelier/pull/11))

### Changed

- Marker selection moved from `CandlestickChart` to `CandleSeries` ([#9](https://github.com/jonathan-g-davis/chandelier/pull/9))

### Other

- Refactored rendering engine ([#8](https://github.com/jonathan-g-davis/chandelier/pull/8))

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
