//! Financial charting widgets for [Ratatui](https://ratatui.rs).

pub mod scale;

mod series;

pub use series::{Candle, price_bounds};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::Widget;

/// A static candlestick chart over a slice of [`Candle`]s.
pub struct Candlestick<'a> {
    candles: &'a [Candle],
}

impl<'a> Candlestick<'a> {
    /// Creates a chart over `candles`.
    #[must_use]
    pub fn new(candles: &'a [Candle]) -> Self {
        Self { candles }
    }
}

impl Widget for &Candlestick<'_> {
    fn render(self, area: Rect, _buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 || self.candles.is_empty() {}
    }
}

impl Widget for Candlestick<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}
