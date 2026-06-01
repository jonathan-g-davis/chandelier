//! Rendering infrastructure.
//!
//! Chart containers compute plot layouts, series generate geometry, and
//! rendering backends quantize it to a glyph family.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

use crate::block::CandleMarks;
use crate::scale::{PriceScale, TimeScale};

/// The laid-out plot area together with the scales mapping data onto it.
///
/// A container computes this once for the drawn area and shares it with
/// downstream components such as series, labels, and overlays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotLayout {
    /// The rectangle the data is drawn into, excluding any axis gutters.
    pub plot: Rect,
    /// Maps prices onto the rows of `plot`.
    pub price: PriceScale,
    /// Maps candle indices onto the columns of `plot`.
    pub time: TimeScale,
    /// The color the plot was filled with, which partial cells blend against.
    pub bg: Color,
}

/// A backend that paints fractional-row geometry into terminal cells.
///
/// Receives raster geometry and the backend quantizes it to the vertical
/// resolution of its glyphs.
pub(crate) trait Rasterizer {
    /// Draws one candle's geometry into `plot`.
    fn draw_candle(&self, buf: &mut Buffer, plot: Rect, marks: &CandleMarks);
}

/// A dataset that knows how to draw itself into a laid-out plot.
///
/// A series produces fractional-row geometry and colors and paints it through a
/// [`Rasterizer`].
pub(crate) trait Series {
    /// The price span the data occupies, or `None` when there is nothing to
    /// draw. The container autoscales the price axis from this.
    fn price_bounds(&self) -> Option<(f64, f64)>;

    /// Lays out this series' columns into `plot`, choosing which entries are in
    /// view.
    fn time_scale(&self, plot: Rect) -> TimeScale;

    /// Draws the visible data into the plot area through a rasterizer.
    fn draw(&self, buf: &mut Buffer, layout: &PlotLayout, rasterizer: &dyn Rasterizer);
}
