//! Annotations layered on top of a chart's series.
//!
//! An overlay shares the chart's plot geometry through its [`PlotLayout`], so it
//! lands on the same rows and columns as the candles it sits over. Overlays are
//! added to a chart with its `overlay` builder and drawn in the order they are
//! added, after the series and before the axes.

mod value_line;

pub use value_line::{LabelSide, LineStyle, ValueLine};

use ratatui_core::buffer::Buffer;

use crate::render::PlotLayout;

/// Annotations or data drawn on top of a chart's series, sharing its plot geometry.
pub(crate) trait OverlayDraw {
    /// The value extent this overlay occupies, or `None` when it should not
    /// affect the chart's value axis. The chart unions this with the series'
    /// bounds when autoscaling, so an overlay stays in view by default.
    fn value_bounds(&self) -> Option<(f64, f64)>;

    /// Draws the overlay into the plot, after the series and before the axes.
    fn draw(&self, buf: &mut Buffer, layout: &PlotLayout);
}

/// An annotation layered over a chart, added with its `overlay` builder.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Overlay<'a> {
    /// A horizontal reference line at a fixed value.
    Value(ValueLine<'a>),
}

impl Overlay<'_> {
    pub(crate) fn value_bounds(&self) -> Option<(f64, f64)> {
        match self {
            Overlay::Value(o) => o.value_bounds(),
        }
    }

    pub(crate) fn draw(&self, buf: &mut Buffer, layout: &PlotLayout) {
        match self {
            Overlay::Value(o) => o.draw(buf, layout),
        }
    }
}

impl<'a> From<ValueLine<'a>> for Overlay<'a> {
    fn from(line: ValueLine<'a>) -> Self {
        Overlay::Value(line)
    }
}

/// Unions the value extents of `overlays` into `base`, so the chart autoscales
/// to include any overlay that opts into it.
pub(crate) fn union_bounds(base: (f64, f64), overlays: &[Overlay<'_>]) -> (f64, f64) {
    let (mut lo, mut hi) = base;
    for overlay in overlays {
        if let Some((olo, ohi)) = overlay.value_bounds() {
            lo = lo.min(olo);
            hi = hi.max(ohi);
        }
    }
    (lo, hi)
}

/// Draws every overlay in order, after the series and before the axes.
pub(crate) fn draw_all(overlays: &[Overlay<'_>], buf: &mut Buffer, layout: &PlotLayout) {
    for overlay in overlays {
        overlay.draw(buf, layout);
    }
}
