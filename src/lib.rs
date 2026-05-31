//! Financial charting widgets for [Ratatui](https://ratatui.rs).

pub mod axis;
pub mod scale;

mod block;
mod candlestick;
mod series;

pub use candlestick::Candlestick;
pub use series::{Candle, price_bounds};
