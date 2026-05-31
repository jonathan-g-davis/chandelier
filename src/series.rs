//! Price data the chart renders.
//!
//! Input is plain values. Chandelier does not fetch, compute, or persist
//! anything. Callers pass already-computed OHLC data.

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
}
