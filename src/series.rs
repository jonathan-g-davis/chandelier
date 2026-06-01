//! Price data the chart renders.
//!
//! Input is plain values. Chandelier does not fetch, compute, or persist
//! anything. Callers pass already-computed OHLC data.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Style};

use crate::render::{CandleGeometry, PlotLayout, Rasterizer, Series};
use crate::scale::TimeScale;

/// A single open/high/low/close bar.
///
/// Prices are `f64`. Time is intentionally absent: a bar's position on the
/// x-axis is its index in the slice handed to the chart, so callers are free to
/// use any time representation (or none).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    /// Opening price.
    pub open: f64,
    /// Highest traded price.
    pub high: f64,
    /// Lowest traded price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
}

impl Candle {
    /// Creates a candle from its four prices.
    pub const fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }

    /// `true` when the bar closed at or above its open (drawn with the bull color).
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    /// The higher of open/close, the top edge of the body.
    pub fn body_top(&self) -> f64 {
        self.open.max(self.close)
    }

    /// The lower of open/close, the bottom edge of the body.
    pub fn body_bottom(&self) -> f64 {
        self.open.min(self.close)
    }
}

/// The lowest low and highest high across a set of candles.
///
/// Returns `None` for an empty slice. Used by the chart to autoscale the price
/// axis to the data in view.
pub fn price_bounds(candles: &[Candle]) -> Option<(f64, f64)> {
    let mut iter = candles.iter();
    let first = iter.next()?;
    let mut lo = first.low;
    let mut hi = first.high;
    for c in iter {
        if c.low < lo {
            lo = c.low;
        }
        if c.high > hi {
            hi = c.high;
        }
    }
    Some((lo, hi))
}

/// A series of candles together with how it is drawn.
///
/// This is the dataset a [`CandlestickChart`](crate::CandlestickChart) renders:
/// the bars plus their bull/bear/wick styles and the column geometry. It owns no
/// axes or scale; the chart supplies those when it draws the series.
#[derive(Debug, Clone)]
pub struct CandleSeries<'a> {
    pub(crate) candles: &'a [Candle],
    bull: Style,
    bear: Style,
    wick: Option<Style>,
    pub(crate) width: u16,
    pub(crate) gap: u16,
}

impl<'a> CandleSeries<'a> {
    /// Creates a series over `candles` with the default green-up / red-down
    /// scheme, three-column bodies, and a one-column gap.
    pub fn new(candles: &'a [Candle]) -> Self {
        Self {
            candles,
            bull: Style::new().fg(Color::Green),
            bear: Style::new().fg(Color::Red),
            wick: None,
            width: 3,
            gap: 1,
        }
    }

    /// Sets the style for bull (close at or above open) bodies. Its foreground
    /// is the body color.
    #[must_use]
    pub fn bull_style(mut self, style: impl Into<Style>) -> Self {
        self.bull = style.into();
        self
    }

    /// Sets the style for bear (close below open) bodies. Its foreground is the
    /// body color.
    #[must_use]
    pub fn bear_style(mut self, style: impl Into<Style>) -> Self {
        self.bear = style.into();
        self
    }

    /// Sets an explicit wick style. Without one, a wick takes its candle's body
    /// color.
    #[must_use]
    pub fn wick_style(mut self, style: impl Into<Style>) -> Self {
        self.wick = Some(style.into());
        self
    }

    /// Sets the candle body width in columns (clamped to at least one when drawn).
    #[must_use]
    pub fn width(mut self, cols: u16) -> Self {
        self.width = cols;
        self
    }

    /// Sets the gap, in columns, between adjacent candles.
    #[must_use]
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// The body color for a candle, taken from the bull or bear style foreground.
    pub(crate) fn body_color(&self, candle: Candle) -> Color {
        let style = if candle.is_bullish() {
            self.bull
        } else {
            self.bear
        };
        style.fg.unwrap_or(Color::Reset)
    }

    /// The wick color for a candle, honoring an explicit wick style and falling
    /// back to the body color.
    pub(crate) fn wick_color(&self, candle: Candle) -> Color {
        self.wick
            .and_then(|w| w.fg)
            .unwrap_or_else(|| self.body_color(candle))
    }
}

impl Series for CandleSeries<'_> {
    fn price_bounds(&self) -> Option<(f64, f64)> {
        price_bounds(self.candles)
    }

    fn time_scale(&self, plot: Rect) -> TimeScale {
        TimeScale::new(plot.width, self.candles.len(), self.width.max(1), self.gap)
    }

    fn draw(&self, buf: &mut Buffer, layout: &PlotLayout, rasterizer: &dyn Rasterizer) {
        let plot = layout.plot;
        let scale = layout.price;
        let time = layout.time;
        let bg = layout.bg;

        for vi in 0..time.visible() {
            let candle = self.candles[time.first_visible() + vi];
            let col_left = plot.x + time.index_to_col(vi);
            let body_cols = time.candle_width();

            let geometry = CandleGeometry {
                cols: col_left..(col_left + body_cols),
                center_col: plot.x + time.index_to_center_col(vi),
                body_top_row: scale.price_to_row_f64(candle.body_top()),
                body_bottom_row: scale.price_to_row_f64(candle.body_bottom()),
                high_row: scale.price_to_row_f64(candle.high),
                low_row: scale.price_to_row_f64(candle.low),
                body: self.body_color(candle),
                wick: self.wick_color(candle),
                bg,
            };
            rasterizer.draw_candle(buf, plot, &geometry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullish_when_close_at_or_above_open() {
        assert!(Candle::new(100.0, 105.0, 99.0, 104.0).is_bullish());
        assert!(Candle::new(100.0, 105.0, 99.0, 100.0).is_bullish());
        assert!(!Candle::new(100.0, 105.0, 95.0, 96.0).is_bullish());
    }

    #[test]
    fn body_edges_are_ordered_regardless_of_direction() {
        let bull = Candle::new(100.0, 110.0, 99.0, 108.0);
        assert_eq!(bull.body_top(), 108.0);
        assert_eq!(bull.body_bottom(), 100.0);

        let bear = Candle::new(108.0, 109.0, 95.0, 100.0);
        assert_eq!(bear.body_top(), 108.0);
        assert_eq!(bear.body_bottom(), 100.0);
    }

    #[test]
    fn price_bounds_spans_lowest_low_and_highest_high() {
        let candles = [
            Candle::new(100.0, 106.0, 98.0, 105.0),
            Candle::new(105.0, 112.0, 104.0, 110.0),
            Candle::new(110.0, 111.0, 90.0, 92.0),
        ];
        assert_eq!(price_bounds(&candles), Some((90.0, 112.0)));
    }

    #[test]
    fn price_bounds_is_none_for_empty() {
        assert_eq!(price_bounds(&[]), None);
    }

    #[test]
    fn series_has_green_up_red_down_defaults() {
        let candles = [
            Candle::new(100.0, 110.0, 99.0, 108.0), // bull
            Candle::new(108.0, 109.0, 95.0, 96.0),  // bear
        ];
        let series = CandleSeries::new(&candles);
        assert_eq!(series.width, 3);
        assert_eq!(series.gap, 1);
        assert_eq!(series.body_color(candles[0]), Color::Green);
        assert_eq!(series.body_color(candles[1]), Color::Red);
    }

    #[test]
    fn wick_color_falls_back_to_body_then_honors_override() {
        let candles = [Candle::new(100.0, 110.0, 99.0, 108.0)]; // bull
        let series = CandleSeries::new(&candles).bull_style(Color::Cyan);
        assert_eq!(series.wick_color(candles[0]), Color::Cyan);

        let series = series.wick_style(Color::Gray);
        assert_eq!(series.wick_color(candles[0]), Color::Gray);
    }
}
