//! A horizontal reference line at a fixed value.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Alignment;
use ratatui_core::style::{Color, Modifier, Style};

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

/// A horizontal line drawn across the whole plot at a fixed value.
///
/// Useful for a reference level such as support or resistance, the last price,
/// or a volume threshold. The line spans the full plot width at the row its
/// value maps to. It can also have an optional label, aligned to the left,
/// center, or right of the line. By default, the chart expands its value axis
/// so the line stays in view.
#[derive(Debug, Clone)]
pub struct ValueLine<'a> {
    value: f64,
    style: Style,
    line: LineStyle,
    label: Option<&'a str>,
    label_alignment: Alignment,
    label_inset: u16,
    label_padding: u16,
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
            label_alignment: Alignment::Right,
            label_inset: 0,
            label_padding: 1,
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

    /// Sets how the label is aligned along the line: against the left edge,
    /// centered, or against the right edge (next to the value axis). Defaults to
    /// [`Alignment::Right`].
    #[must_use]
    pub fn label_alignment(mut self, alignment: Alignment) -> Self {
        self.label_alignment = alignment;
        self
    }

    /// Sets how many columns of line lead in from the aligned edge before the
    /// label, so a left- or right-aligned label sits inset in the line rather
    /// than flush against the edge (`──RESISTANCE────`). Ignored when the label
    /// is centered. Defaults to `0`.
    #[must_use]
    pub fn label_inset(mut self, inset: u16) -> Self {
        self.label_inset = inset;
        self
    }

    /// Sets how many blank columns separate the label from the line on each
    /// side, so the line breaks cleanly around the text. Defaults to `1`.
    #[must_use]
    pub fn label_padding(mut self, padding: u16) -> Self {
        self.label_padding = padding;
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
        let y = plot.y + row;
        let glyph = match self.line {
            LineStyle::Solid => "─",
            LineStyle::Dashed => "╌",
        };

        // Paint on the chart background so the line and label read as opaque
        // marks over the candles. Clearing REVERSED keeps a candle cell's
        // foreground/background inversion from swapping the colors we set.
        let bg = self.style.bg.unwrap_or(layout.bg);
        let style = self.style.bg(bg).remove_modifier(Modifier::REVERSED);

        for col in 0..plot.width {
            render::put(buf, plot, u32::from(col), u32::from(row), glyph, style);
        }

        if let Some(label) = self.label {
            let len = label.chars().count() as u16;
            let right = plot.x + plot.width;
            let pad = self.label_padding;
            let inset = self.label_inset;
            let max_start = right.saturating_sub(len).max(plot.x);

            // `inset` columns of line lead in from the aligned edge before the
            // label. A padding gap separates the label from the line on each side
            // that carries line: the inner side always, and the lead-in (edge)
            // side only when the label is inset. A flush label keeps no gap
            // against its edge. The lead-in columns themselves stay as line.
            let edge_gap = if inset > 0 { pad } else { 0 };
            let (start, gap_start, gap_end) = match self.label_alignment {
                Alignment::Left => {
                    let start = (plot.x + inset + edge_gap).min(max_start);
                    (start, plot.x + inset, start + len + pad)
                }
                Alignment::Right => {
                    let start = right
                        .saturating_sub(inset)
                        .saturating_sub(edge_gap)
                        .saturating_sub(len)
                        .clamp(plot.x, max_start);
                    (
                        start,
                        start.saturating_sub(pad),
                        right.saturating_sub(inset),
                    )
                }
                Alignment::Center => {
                    let start = (plot.x + plot.width.saturating_sub(len) / 2).min(max_start);
                    (start, start.saturating_sub(pad), start + len + pad)
                }
            };

            // Break the line by clearing the padding gaps to the background,
            // then draw the label on top.
            let blank = Style::new().bg(bg).remove_modifier(Modifier::REVERSED);
            for cx in gap_start.max(plot.x)..gap_end.min(right) {
                buf.set_string(cx, y, " ", blank);
            }
            buf.set_string(start, y, label, style);
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
    fn defaults_are_solid_right_aligned_and_unlabeled() {
        let line = ValueLine::at(1.0);
        assert_eq!(line.line, LineStyle::Solid);
        assert_eq!(line.label_alignment, Alignment::Right);
        assert_eq!(line.label_inset, 0);
        assert_eq!(line.label_padding, 1);
        assert!(line.label.is_none());
        assert!(line.autoscale);
    }
}
