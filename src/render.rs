//! Rendering infrastructure.
//!
//! Chart containers compute plot layouts, series generate geometry, and
//! rendering backends quantize it to a glyph family.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

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

/// Whether a candle body is drawn solid or as an outline.
///
/// - `Filled` paints the whole body.
/// - `Hollow` traces the body's border and leaves the interior empty.
///
/// A body too small to enclose an interior is drawn filled, depending on the
/// marker that is used to render the chart.
///
/// Set per direction on a [`CandleSeries`](crate::CandleSeries) with
/// [`bull_fill`](crate::CandleSeries::bull_fill) and
/// [`bear_fill`](crate::CandleSeries::bear_fill).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BodyFill {
    /// A solid body.
    #[default]
    Filled,
    /// An outlined body with an empty interior.
    Hollow,
}

/// One candle's geometry and colors.
///
/// Geometry is defined in terms of plot area, not a specific glyph family. Both
/// axes are continuous and measured from the plot's top-left corner: the body
/// spans the fractional columns `[body_left, body_right)` and the four row
/// fields are fractional rows (smaller is higher on screen). A backend quantizes
/// each axis to its own grid, so the same geometry draws through any glyph
/// family. The wick runs along the body's horizontal [`center`](Self::center).
pub(crate) struct CandleGeometry {
    /// The fractional column of the left edge of the body.
    pub body_left: f64,
    /// The fractional column of the right edge of the body.
    pub body_right: f64,
    /// The fractional row of the top of the body.
    pub body_top_row: f64,
    /// The fractional row of the bottom of the body.
    pub body_bottom_row: f64,
    /// The fractional row of the high wick.
    pub high_row: f64,
    /// The fractional row of the low wick.
    pub low_row: f64,
    /// The color of the body.
    pub body: Color,
    /// The color of the wick.
    pub wick: Color,
    /// The color the empty portion of a partially filled cell is painted.
    pub bg: Color,
    /// Whether the body is solid or an outline.
    pub fill: BodyFill,
}

impl CandleGeometry {
    /// The fractional column at the horizontal center of the body, where the
    /// wick is drawn. A backend quantizes this to the nearest sub-cell.
    pub(crate) fn center(&self) -> f64 {
        (self.body_left + self.body_right) / 2.0
    }
}

/// A backend that paints fractional-row geometry into terminal cells.
///
/// Receives raster geometry and the backend quantizes it to the vertical
/// resolution of its glyphs.
pub(crate) trait Rasterizer {
    /// Draws one candle's geometry into `plot`.
    fn draw_candle(&self, buf: &mut Buffer, plot: Rect, geometry: &CandleGeometry);
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
