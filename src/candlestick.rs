//! The candlestick chart widget.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Style, Styled};
use ratatui_core::widgets::Widget;

use crate::axis;
use crate::block::{self, CandleMarks};
use crate::scale::{PriceScale, TimeScale};
use crate::series::{Candle, price_bounds};

/// A static candlestick chart over a slice of [`Candle`]s.
///
/// The chart autoscales the price axis to the data in view and draws the most
/// recent candles that fit, right-aligned. Body endpoints are placed to the
/// nearest eighth of a row, so open and close levels do not snap to whole rows.
///
/// Colors are set with [`Style`] objects: a base style for the chart (its
/// background is what the partial-cell rendering blends against, so set it to
/// your terminal's background for a crisp top edge) plus per-role styles for
/// bull and bear bodies, wicks, and axis labels. The widget implements
/// [`Styled`], so [`Stylize`](ratatui_core::style::Stylize) shorthands set the
/// base style.
#[derive(Debug, Clone)]
pub struct Candlestick<'a> {
    candles: &'a [Candle],
    base: Style,
    bull: Style,
    bear: Style,
    wick: Option<Style>,
    axis: Style,
    width: u16,
    gap: u16,
    pad_frac: f64,
    price_axis_width: u16,
    x_labels: Option<&'a [String]>,
    show_axes: bool,
}

impl<'a> Candlestick<'a> {
    /// Creates a chart over `candles` with the default green-up / red-down scheme.
    pub fn new(candles: &'a [Candle]) -> Self {
        Self {
            candles,
            base: Style::new(),
            bull: Style::new().fg(Color::Green),
            bear: Style::new().fg(Color::Red),
            wick: None,
            axis: Style::new().fg(Color::Gray),
            width: 3,
            gap: 1,
            pad_frac: 0.05,
            price_axis_width: 8,
            x_labels: None,
            show_axes: true,
        }
    }

    /// Sets the candle body width in columns (clamped to at least one).
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

    /// Sets the base style. Its background is the color partial cells blend
    /// against, so set it to your terminal background for a crisp body top edge.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.base = style.into();
        self
    }

    /// Sets the style for bull (close at or above open) bodies. The style's
    /// foreground is the body color.
    #[must_use]
    pub fn bull_style(mut self, style: impl Into<Style>) -> Self {
        self.bull = style.into();
        self
    }

    /// Sets the style for bear (close below open) bodies. The style's foreground
    /// is the body color.
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

    /// Sets the style for axis labels.
    #[must_use]
    pub fn axis_style(mut self, style: impl Into<Style>) -> Self {
        self.axis = style.into();
        self
    }

    /// Sets the autoscale padding as a fraction of the price span (per end).
    #[must_use]
    pub fn padding(mut self, pad_frac: f64) -> Self {
        self.pad_frac = pad_frac;
        self
    }

    /// Sets the width, in columns, reserved for the right-hand price axis.
    #[must_use]
    pub fn price_axis_width(mut self, cols: u16) -> Self {
        self.price_axis_width = cols;
        self
    }

    /// Supplies x-axis labels aligned to the full candle slice (index `i` labels
    /// `candles[i]`). Without labels, the time axis shows candle indices.
    #[must_use]
    pub fn x_labels(mut self, labels: &'a [String]) -> Self {
        self.x_labels = Some(labels);
        self
    }

    /// Shows or hides the price and time axes.
    #[must_use]
    pub fn axes(mut self, show: bool) -> Self {
        self.show_axes = show;
        self
    }

    /// The body color for a candle, taken from the bull or bear style foreground.
    fn body_color(&self, candle: Candle) -> Color {
        let style = if candle.is_bullish() {
            self.bull
        } else {
            self.bear
        };
        style.fg.unwrap_or(Color::Reset)
    }

    /// The wick color for a candle, honoring an explicit wick style and falling
    /// back to the body color.
    fn wick_color(&self, candle: Candle) -> Color {
        self.wick
            .and_then(|w| w.fg)
            .unwrap_or_else(|| self.body_color(candle))
    }

    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 || self.candles.is_empty() {
            return;
        }

        let bg = self.base.bg.unwrap_or(Color::Reset);
        let fill = self.base.bg(bg);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_style(fill);
                }
            }
        }

        let right_axis_w = if self.show_axes {
            self.price_axis_width
        } else {
            0
        };
        let bottom_axis_h = if self.show_axes { 1 } else { 0 };

        if area.width <= right_axis_w || area.height <= bottom_axis_h {
            return;
        }

        let plot = Rect {
            x: area.x,
            y: area.y,
            width: area.width - right_axis_w,
            height: area.height - bottom_axis_h,
        };

        let Some((lo, hi)) = price_bounds(self.candles) else {
            return;
        };
        let scale = PriceScale::autoscale(lo, hi, plot.height, self.pad_frac);
        let time = TimeScale::new(plot.width, self.candles.len(), self.width.max(1), self.gap);

        for vi in 0..time.visible() {
            let candle = self.candles[time.first_visible() + vi];
            let col_left = plot.x + time.index_to_col(vi);
            let body_cols = time.candle_width();

            let marks = CandleMarks {
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
            block::draw_candle(buf, plot, &marks);
        }

        if self.show_axes {
            self.draw_price_axis(buf, &scale, plot, right_axis_w);
            self.draw_time_axis(buf, &time, plot);
        }
    }

    fn draw_price_axis(&self, buf: &mut Buffer, scale: &PriceScale, plot: Rect, axis_w: u16) {
        let ticks = axis::price_ticks(scale.min(), scale.max(), 6);
        let step = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        let axis_x = plot.x + plot.width;
        let width = axis_w as usize;

        for &t in ticks.iter() {
            if t < scale.min() || t > scale.max() {
                continue;
            }

            let row = scale.price_to_row(t);
            let label = axis::format_price(t, step);
            let padded = format!("{label:>width$}");
            buf.set_string(axis_x, plot.y + row, padded, self.axis);
        }
    }

    fn draw_time_axis(&self, buf: &mut Buffer, time: &TimeScale, plot: Rect) {
        let y = plot.y + plot.height;
        let mut next_free: u16 = plot.x;

        for vi in 0..time.visible() {
            let orig = time.first_visible() + vi;
            let label = match self.x_labels {
                Some(labels) if orig < labels.len() => labels[orig].clone(),
                _ => orig.to_string(),
            };
            let cx = plot.x + time.index_to_center_col(vi);
            let len = label.chars().count() as u16;

            if cx >= next_free && cx + len <= plot.x + plot.width {
                buf.set_string(cx, y, &label, self.axis);
                next_free = cx + len + 2;
            }
        }
    }
}

impl Widget for &Candlestick<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_chart(area, buf);
    }
}

impl Widget for Candlestick<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_chart(area, buf);
    }
}

impl<'a> Styled for Candlestick<'a> {
    type Item = Candlestick<'a>;

    fn style(&self) -> Style {
        self.base
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.base = style.into();
        self
    }
}
