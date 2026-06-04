//! An index-aligned line over the candles, such as a moving average.

use ratatui_core::buffer::Buffer;
use ratatui_core::style::{Color, Style};

use crate::marker::Marker;
use crate::overlay::OverlayDraw;
use crate::render::PlotLayout;

/// A connected line over the candles, drawn from values aligned to them.
#[derive(Debug, Clone)]
pub struct LineOverlay<'a> {
    values: &'a [Option<f64>],
    style: Style,
    marker: Marker,
    autoscale: bool,
}

impl<'a> LineOverlay<'a> {
    /// Creates a white braille line from values aligned one-to-one with the
    /// chart's candles, where `None` breaks the line.
    pub fn new(values: &'a [Option<f64>]) -> Self {
        Self {
            values,
            style: Style::new().fg(Color::White),
            marker: Marker::Braille,
            autoscale: true,
        }
    }

    /// Sets the line style. Its foreground is the line color.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the glyph family the line is drawn with. Defaults to
    /// [`Marker::Braille`].
    ///
    /// [`Block`](Marker::Block) and [`BoxDrawing`](Marker::BoxDrawing) fall back
    /// to [`Quadrant`](Marker::Quadrant).
    #[must_use]
    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    /// Sets whether the chart expands its value axis to keep this line in view.
    /// On by default.
    #[must_use]
    pub fn autoscale(mut self, autoscale: bool) -> Self {
        self.autoscale = autoscale;
        self
    }
}

impl OverlayDraw for LineOverlay<'_> {
    fn value_bounds(&self) -> Option<(f64, f64)> {
        if !self.autoscale {
            return None;
        }
        let mut iter = self.values.iter().flatten().copied();
        let first = iter.next()?;
        let (mut lo, mut hi) = (first, first);
        for value in iter {
            lo = lo.min(value);
            hi = hi.max(value);
        }
        Some((lo, hi))
    }

    fn draw(&self, buf: &mut Buffer, layout: &PlotLayout) {
        let plot = layout.plot;
        if plot.width == 0 || plot.height == 0 {
            return;
        }

        let time = &layout.time;
        let scale = &layout.value;
        let color = self.style.fg.unwrap_or(Color::Reset);
        let bg = self.style.bg.unwrap_or(layout.bg);

        let points: Vec<Option<(f64, f64)>> = (0..time.visible())
            .map(|vi| {
                let value = (*self.values.get(time.first_visible() + vi)?)?;
                let col = time.index_to_left(vi) + time.candle_width() / 2.0;
                Some((col, scale.value_to_row_f64(value)))
            })
            .collect();

        self.marker
            .line_rasterizer()
            .draw_polyline(buf, plot, &points, color, bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_bounds_span_some_values_and_ignore_none() {
        let values = [None, Some(10.0), Some(30.0), None, Some(20.0)];
        assert_eq!(LineOverlay::new(&values).value_bounds(), Some((10.0, 30.0)));
        assert_eq!(
            LineOverlay::new(&values).autoscale(false).value_bounds(),
            None
        );
        assert_eq!(LineOverlay::new(&[None, None]).value_bounds(), None);
        assert_eq!(LineOverlay::new(&[]).value_bounds(), None);
    }
}
