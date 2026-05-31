//! The coordinate substrate every renderer draws through.
//!
//! A [`PriceScale`] maps a price domain onto the rows of a plot area and a
//! [`TimeScale`] maps candle indices onto columns. Both directions of each map
//! are exposed so a price (or column) can be turned back into the other so that
//! a crosshair or inspection readout can be displayed.
//!
//! The price map works in continuous fractional rows rather than whole rows or
//! a fixed sub-cell grid. A renderer asks where a price lands as a fractional
//! row and quantizes that to whatever vertical resolution its glyphs offer, so
//! the scale stays independent of the character set used to draw.

/// Maps a price domain `[min, max]` onto a plot `height` (in rows), in both
/// directions, at continuous (fractional-row) resolution.
///
/// Row `0.0` is the top of the plot and higher prices map to smaller row
/// values, matching screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceScale {
    min: f64,
    max: f64,
    height: u16,
}

impl PriceScale {
    /// Builds a scale over `[min, max]` for a plot `height` in rows.
    ///
    /// A zero or inverted span is widened to a small non-zero range so a flat
    /// series still renders instead of dividing by zero.
    pub fn new(min: f64, max: f64, height: u16) -> Self {
        let (min, max) = if (max - min).abs() < f64::EPSILON {
            (min - 1.0, max + 1.0)
        } else {
            (min, max)
        };
        Self {
            min,
            max,
            height: height.max(1),
        }
    }

    /// Builds a scale from the candles' price bounds, padded by `pad_frac` of
    /// the span on each end (e.g. `0.05` for 5% headroom top and bottom).
    pub fn autoscale(min: f64, max: f64, height: u16, pad_frac: f64) -> Self {
        let pad = (max - min).abs() * pad_frac;
        Self::new(min - pad, max + pad, height)
    }

    /// Lowest price in the domain.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Highest price in the domain.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Plot height in rows.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Maps a price to a fractional row measured from the top of the plot,
    /// clamped to the plot. Smaller values are higher on screen.
    pub fn price_to_row_f64(&self, price: f64) -> f64 {
        let frac = (price - self.min) / (self.max - self.min);
        let from_top = (1.0 - frac) * f64::from(self.height);
        from_top.clamp(0.0, f64::from(self.height))
    }

    /// Inverse of [`price_to_row_f64`](Self::price_to_row_f64): turns a fractional
    /// row back into a price. Not clamped, so it round-trips prices outside the
    /// domain as well.
    pub fn row_f64_to_price(&self, row_f: f64) -> f64 {
        let frac = 1.0 - row_f / f64::from(self.height);
        self.min + frac * (self.max - self.min)
    }

    /// Maps a price to its whole terminal row within the plot.
    pub fn price_to_row(&self, price: f64) -> u16 {
        (self.price_to_row_f64(price) as u16).min(self.height - 1)
    }
}

/// Maps candle indices to columns of the plot, in both directions.
///
/// Each candle occupies `candle_width` columns followed by a `gap` of empty
/// columns. The most recent candles that fit are shown, right-aligned, so a
/// growing series scrolls like a real chart instead of overflowing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScale {
    width: u16,
    candle_width: u16,
    gap: u16,
    /// Index of the leftmost visible candle in the full series.
    first_visible: usize,
    /// Number of visible candles.
    visible: usize,
}

impl TimeScale {
    /// Lays out `candle_count` candles into a plot `width`, drawing each
    /// `candle_width` columns wide with `gap` columns between them.
    pub fn new(width: u16, candle_count: usize, candle_width: u16, gap: u16) -> Self {
        let candle_width = candle_width.max(1);
        let slot = u32::from(candle_width) + u32::from(gap);
        // The final candle needs no trailing gap. `slot` is always >= 1 because
        // `candle_width` is clamped to at least 1.
        let capacity = ((u32::from(width) + u32::from(gap)) / slot) as usize;
        let visible = capacity.min(candle_count);
        let first_visible = candle_count - visible;
        Self {
            width,
            candle_width,
            gap,
            first_visible,
            visible,
        }
    }

    /// Index of the leftmost visible candle in the original series.
    pub fn first_visible(&self) -> usize {
        self.first_visible
    }

    /// How many candles are visible.
    pub fn visible(&self) -> usize {
        self.visible
    }

    /// Width drawn for each candle body, in columns.
    pub fn candle_width(&self) -> u16 {
        self.candle_width
    }

    /// The leftmost column of the visible candle at `visible_index`
    /// (`0` is the leftmost visible candle).
    pub fn index_to_col(&self, visible_index: usize) -> u16 {
        let slot = self.candle_width + self.gap;
        (visible_index as u16).saturating_mul(slot)
    }

    /// Center column of the visible candle at `visible_index`, where the wick
    /// is drawn.
    pub fn index_to_center_col(&self, visible_index: usize) -> u16 {
        self.index_to_col(visible_index) + self.candle_width / 2
    }

    /// Inverse of [`index_to_col`](Self::index_to_col): the visible candle a
    /// column falls in, or `None` for a gap or out-of-range column.
    pub fn col_to_index(&self, col: u16) -> Option<usize> {
        if col >= self.width {
            return None;
        }
        let slot = self.candle_width + self.gap;
        let idx = (col / slot) as usize;
        let within = col % slot;
        if within < self.candle_width && idx < self.visible {
            Some(idx)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_round_trips_through_row_f() {
        let scale = PriceScale::new(100.0, 200.0, 20);
        for price in [100.0, 125.0, 150.0, 199.0] {
            let row = scale.price_to_row_f64(price);
            let back = scale.row_f64_to_price(row);
            assert!((price - back).abs() < 1e-9, "{price} -> {row} -> {back}");
        }
    }

    #[test]
    fn higher_prices_map_to_smaller_rows() {
        let scale = PriceScale::new(0.0, 100.0, 10);
        assert!(scale.price_to_row_f64(90.0) < scale.price_to_row_f64(10.0));
        assert_eq!(scale.price_to_row(100.0), 0);
        assert_eq!(scale.price_to_row(0.0), 9);
    }

    #[test]
    fn flat_series_does_not_divide_by_zero() {
        let scale = PriceScale::new(50.0, 50.0, 10);
        let row = scale.price_to_row_f64(50.0);
        assert!(row.is_finite());
    }

    #[test]
    fn column_round_trips_through_index() {
        let time = TimeScale::new(40, 8, 3, 1);
        for vi in 0..time.visible() {
            let col = time.index_to_col(vi);
            assert_eq!(time.col_to_index(col), Some(vi));
        }
        // A gap column maps to no candle.
        let gap_col = time.index_to_col(0) + 3; // just past a 3-wide body
        assert_eq!(time.col_to_index(gap_col), None);
    }

    #[test]
    fn shows_most_recent_candles_when_space_is_tight() {
        // Room for only a few candles out of many: the latest are kept.
        let time = TimeScale::new(12, 100, 3, 1);
        assert!(time.visible() < 100);
        assert_eq!(time.first_visible() + time.visible(), 100);
    }
}
