//! The candlestick chart widget.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Style, Styled};
use ratatui_core::widgets::Widget;

use crate::axis::{self, PriceAxis, TimeAxis};
use crate::block::{self, CandleMarks};
use crate::scale::{PriceScale, TimeScale};
use crate::series::{CandleSeries, price_bounds};

/// A candlestick chart: a [`CandleSeries`] drawn with a [`PriceAxis`] and a
/// [`TimeAxis`].
///
/// The chart autoscales the price axis to the data in view and draws the most
/// recent candles that fit, right-aligned. Body endpoints are placed to the
/// nearest eighth of a row, so open and close levels do not snap to whole rows.
///
/// The chart's own [`Style`] is the background the partial-cell rendering blends
/// against; set its background to your terminal's for a crisp body top edge. The
/// widget implements [`Styled`], so [`Stylize`](ratatui_core::style::Stylize)
/// shorthands set that base style. Per-candle colors live on the [`CandleSeries`];
/// label colors live on each axis.
#[derive(Debug, Clone)]
pub struct CandlestickChart<'a> {
    series: CandleSeries<'a>,
    base: Style,
    pad_frac: f64,
    price_axis: PriceAxis,
    time_axis: TimeAxis<'a>,
    show_axes: bool,
}

impl<'a> CandlestickChart<'a> {
    /// Creates a chart that draws `series` with default axes and padding.
    pub fn new(series: CandleSeries<'a>) -> Self {
        Self {
            series,
            base: Style::new(),
            pad_frac: 0.05,
            price_axis: PriceAxis::new(),
            time_axis: TimeAxis::new(),
            show_axes: true,
        }
    }

    /// Sets the base style. Its background is the color partial cells blend
    /// against, so set it to your terminal background for a crisp body top edge.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.base = style.into();
        self
    }

    /// Sets the autoscale padding as a fraction of the price span (per end).
    #[must_use]
    pub fn padding(mut self, pad_frac: f64) -> Self {
        self.pad_frac = pad_frac;
        self
    }

    /// Sets the price (vertical) axis.
    #[must_use]
    pub fn price_axis(mut self, axis: PriceAxis) -> Self {
        self.price_axis = axis;
        self
    }

    /// Sets the time (horizontal) axis.
    #[must_use]
    pub fn time_axis(mut self, axis: TimeAxis<'a>) -> Self {
        self.time_axis = axis;
        self
    }

    /// Shows or hides both axes.
    #[must_use]
    pub fn axes(mut self, show: bool) -> Self {
        self.show_axes = show;
        self
    }

    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        let candles = self.series.candles;
        if area.width == 0 || area.height == 0 || candles.is_empty() {
            return;
        }

        // Uniform background so the partial-cell inversion blends seamlessly.
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
            self.price_axis.width
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

        let Some((lo, hi)) = price_bounds(candles) else {
            return;
        };
        let scale = PriceScale::autoscale(lo, hi, plot.height, self.pad_frac);
        let time = TimeScale::new(
            plot.width,
            candles.len(),
            self.series.width.max(1),
            self.series.gap,
        );

        for vi in 0..time.visible() {
            let candle = candles[time.first_visible() + vi];
            let col_left = plot.x + time.index_to_col(vi);
            let body_cols = time.candle_width();

            let marks = CandleMarks {
                cols: col_left..(col_left + body_cols),
                center_col: plot.x + time.index_to_center_col(vi),
                body_top_row: scale.price_to_row_f64(candle.body_top()),
                body_bottom_row: scale.price_to_row_f64(candle.body_bottom()),
                high_row: scale.price_to_row_f64(candle.high),
                low_row: scale.price_to_row_f64(candle.low),
                body: self.series.body_color(candle),
                wick: self.series.wick_color(candle),
                bg,
            };
            block::draw_candle(buf, plot, &marks);
        }

        if self.show_axes {
            self.draw_price_axis(buf, &scale, plot);
            self.draw_time_axis(buf, &time, plot);
        }
    }

    fn draw_price_axis(&self, buf: &mut Buffer, scale: &PriceScale, plot: Rect) {
        let ticks = axis::price_ticks(scale.min(), scale.max(), 6);
        let step = if ticks.len() >= 2 {
            ticks[1] - ticks[0]
        } else {
            1.0
        };
        let axis_x = plot.x + plot.width;
        let width = self.price_axis.width as usize;

        for &t in ticks.iter() {
            if t < scale.min() || t > scale.max() {
                continue;
            }

            let row = scale.price_to_row(t);
            let label = axis::format_price(t, step);
            let padded = format!("{label:>width$}");
            buf.set_string(axis_x, plot.y + row, padded, self.price_axis.style);
        }
    }

    fn draw_time_axis(&self, buf: &mut Buffer, time: &TimeScale, plot: Rect) {
        let y = plot.y + plot.height;
        let mut next_free: u16 = plot.x;

        for vi in 0..time.visible() {
            let orig = time.first_visible() + vi;
            let label = match self.time_axis.labels {
                Some(labels) if orig < labels.len() => labels[orig].clone(),
                _ => orig.to_string(),
            };
            let cx = plot.x + time.index_to_center_col(vi);
            let len = label.chars().count() as u16;

            if cx >= next_free && cx + len <= plot.x + plot.width {
                buf.set_string(cx, y, &label, self.time_axis.style);
                next_free = cx + len + 2;
            }
        }
    }
}

impl Widget for &CandlestickChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_chart(area, buf);
    }
}

impl Widget for CandlestickChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_chart(area, buf);
    }
}

impl<'a> Styled for CandlestickChart<'a> {
    type Item = CandlestickChart<'a>;

    fn style(&self) -> Style {
        self.base
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.base = style.into();
        self
    }
}
