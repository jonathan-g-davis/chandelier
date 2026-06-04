//! A horizontal reference line at a fixed value.

use ratatui_core::buffer::Buffer;
use ratatui_core::style::{Color, Style};

use crate::overlay::OverlayDraw;
use crate::render::{self, PlotLayout};

/// Whether a [`ValueLine`] is drawn solid or dashed.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LineStyle {
    /// A continuous line (`─`).
    #[default]
    Solid,
    /// A broken line (`╌`).
    Dashed,
}

/// Where a [`ValueLine`]'s label sits along the line.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LabelSide {
    /// Right-aligned at the plot's right edge, next to the value axis.
    #[default]
    NearAxis,
    /// Left-aligned at the plot's left edge, over the start of the line.
    Inline,
}

/// A horizontal line drawn across the whole plot at a fixed value.
///
/// Useful for a reference level such as support or resistance, the last price,
/// or a volume threshold. The line spans the full plot width at the row its
/// value maps to. It can also have an optional label displayed next to it. By
/// default, the chart expands its value axis so the line stays in view.
#[derive(Debug, Clone)]
pub struct ValueLine<'a> {
    value: f64,
    style: Style,
    line: LineStyle,
    label: Option<&'a str>,
    label_side: LabelSide,
    autoscale: bool,
}

impl<'a> ValueLine<'a> {
    /// Creates a solid gray line at `value` with no label.
    pub fn at(value: f64) -> Self {
        Self {
            value,
            style: Style::new().fg(Color::Gray),
            line: LineStyle::Solid,
            label: None,
            label_side: LabelSide::NearAxis,
            autoscale: true,
        }
    }

    /// Sets the line style. Its foreground is the line color.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets whether the line is solid or dashed.
    #[must_use]
    pub fn line(mut self, line: LineStyle) -> Self {
        self.line = line;
        self
    }

    /// Draws the line dashed. Shorthand for [`line`](Self::line) with
    /// [`LineStyle::Dashed`].
    #[must_use]
    pub fn dashed(mut self) -> Self {
        self.line = LineStyle::Dashed;
        self
    }

    /// Sets a label naming the line.
    #[must_use]
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets where the label sits along the line.
    #[must_use]
    pub fn label_side(mut self, side: LabelSide) -> Self {
        self.label_side = side;
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

impl OverlayDraw for ValueLine<'_> {
    fn value_bounds(&self) -> Option<(f64, f64)> {
        self.autoscale.then_some((self.value, self.value))
    }

    fn draw(&self, buf: &mut Buffer, layout: &PlotLayout) {
        let plot = layout.plot;
        if plot.width == 0 || plot.height == 0 {
            return;
        }

        let row = layout.value.value_to_row(self.value);
        let glyph = match self.line {
            LineStyle::Solid => "─",
            LineStyle::Dashed => "╌",
        };

        for col in 0..plot.width {
            render::put(buf, plot, u32::from(col), u32::from(row), glyph, self.style);
        }

        if let Some(label) = self.label {
            let len = label.chars().count() as u16;
            let x = match self.label_side {
                LabelSide::NearAxis => (plot.x + plot.width).saturating_sub(len).max(plot.x),
                LabelSide::Inline => plot.x,
            };
            buf.set_string(x, plot.y + row, label, self.style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_bounds_pins_to_the_value_and_honors_autoscale() {
        assert_eq!(ValueLine::at(42.0).value_bounds(), Some((42.0, 42.0)));
        assert_eq!(ValueLine::at(42.0).autoscale(false).value_bounds(), None);
    }

    #[test]
    fn defaults_are_solid_near_axis_and_unlabeled() {
        let line = ValueLine::at(1.0);
        assert_eq!(line.line, LineStyle::Solid);
        assert_eq!(line.label_side, LabelSide::NearAxis);
        assert!(line.label.is_none());
        assert!(line.autoscale);
    }
}
