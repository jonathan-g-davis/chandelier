//! The candlestick chart widget.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Rect};
use ratatui_core::style::{Color, Style, Styled};
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::{Block, BlockExt};

use crate::axis::{self, PriceAxis, TimeAxis};
use crate::render::{PlotLayout, Series};
use crate::scale::{PriceScale, TimeScale};
use crate::series::CandleSeries;

/// A candlestick chart: a [`CandleSeries`] drawn with a [`PriceAxis`] and a
/// [`TimeAxis`].
///
/// The chart autoscales the price axis to the data in view and draws the most
/// recent candles that fit, right-aligned.
#[derive(Debug, Clone)]
pub struct CandlestickChart<'a> {
    block: Option<Block<'a>>,
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
            block: None,
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

    /// Wraps the chart with the given [`Block`]
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.base);

        self.block.as_ref().render(area, buf);
        let chart_area = self.block.inner_if_some(area);

        let bg = self.base.bg.unwrap_or(Color::Reset);
        let Some(layout) = self.layout(chart_area, bg) else {
            return;
        };

        self.series.draw(buf, &layout);
        self.draw_overlays(buf, &layout);

        if self.show_axes {
            self.draw_price_axis(buf, &layout.price, layout.plot);
            self.draw_time_axis(buf, &layout.time, layout.plot);
        }
    }

    /// Computes the plot rectangle and the price and time scales for `area`.
    ///
    /// This is the single place the drawn plot geometry is laid out, so the
    /// series, the axes, and anything aligning to the same columns and rows all
    /// share one [`PlotLayout`]. Returns `None` when no plot fits.
    fn layout(&self, area: Rect, bg: Color) -> Option<PlotLayout> {
        let right_axis_w = if self.show_axes {
            self.price_axis.width
        } else {
            0
        };
        let bottom_axis_h = if self.show_axes { 1 } else { 0 };

        if area.width <= right_axis_w || area.height <= bottom_axis_h {
            return None;
        }

        let plot = Rect {
            x: area.x,
            y: area.y,
            width: area.width - right_axis_w,
            height: area.height - bottom_axis_h,
        };

        let (lo, hi) = self.series.price_bounds()?;
        let price = PriceScale::autoscale(lo, hi, plot.height, self.pad_frac);
        let time = self.series.time_scale(plot);

        Some(PlotLayout {
            plot,
            price,
            time,
            bg,
        })
    }

    /// Draws anything layered on top of the series, after it and before the
    /// axes. Overlays align to `layout`'s scales. There are none by default.
    fn draw_overlays(&self, _buf: &mut Buffer, _layout: &PlotLayout) {}

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
            let padded = match self.price_axis.labels_alignment {
                Alignment::Left => format!("{label:<width$}"),
                Alignment::Center => format!("{label:^width$}"),
                Alignment::Right => format!("{label:>width$}"),
            };
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
            let start = match self.time_axis.labels_alignment {
                Alignment::Left => cx,
                Alignment::Center => cx.saturating_sub(len / 2),
                Alignment::Right => cx.saturating_sub(len.saturating_sub(1)),
            };

            if start >= next_free && start + len <= plot.x + plot.width {
                buf.set_string(start, y, &label, self.time_axis.style);
                next_free = start + len + 2;
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
