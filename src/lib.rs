//! Financial charting widgets for [Ratatui](https://ratatui.rs).

mod axis;
mod block;
mod chart;
mod scale;
mod series;

pub use axis::{PriceAxis, TimeAxis};
pub use chart::CandlestickChart;
pub use scale::{PriceScale, TimeScale};
pub use series::{Candle, CandleSeries, price_bounds};
