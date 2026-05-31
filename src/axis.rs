//! The chart's two axes and the tick selection behind them.
//!
//! [`PriceAxis`] and [`TimeAxis`] are small style-and-layout configurations the
//! chart composes; the tick helpers pick where price labels go and how they read.

use ratatui_core::style::{Color, Style, Styled};

/// Configuration for the price (vertical) axis.
///
/// Carries how the labels are styled and how many columns the axis reserves on
/// the right. The value range and tick positions are chosen automatically from
/// the data in view; choosing them manually is not offered yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceAxis {
    pub(crate) style: Style,
    pub(crate) width: u16,
}

impl PriceAxis {
    /// A price axis with gray labels reserving eight columns.
    pub fn new() -> Self {
        Self {
            style: Style::new().fg(Color::Gray),
            width: 8,
        }
    }

    /// Sets the label style.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the width, in columns, reserved for the axis labels.
    #[must_use]
    pub fn width(mut self, cols: u16) -> Self {
        self.width = cols;
        self
    }
}

impl Default for PriceAxis {
    fn default() -> Self {
        Self::new()
    }
}

impl Styled for PriceAxis {
    type Item = PriceAxis;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

/// Configuration for the time (horizontal) axis.
///
/// Carries the label style and, optionally, the text for each candle aligned to
/// the full series (index `i` labels `candles[i]`). Without labels the axis
/// shows candle indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeAxis<'a> {
    pub(crate) style: Style,
    pub(crate) labels: Option<&'a [String]>,
}

impl<'a> TimeAxis<'a> {
    /// A time axis with gray labels and no explicit label text.
    pub fn new() -> Self {
        Self {
            style: Style::new().fg(Color::Gray),
            labels: None,
        }
    }

    /// Sets the label style.
    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the label text, aligned to the full candle slice.
    #[must_use]
    pub fn labels(mut self, labels: &'a [String]) -> Self {
        self.labels = Some(labels);
        self
    }
}

impl Default for TimeAxis<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Styled for TimeAxis<'a> {
    type Item = TimeAxis<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

/// Rounds a span to a "nice" number (1, 2, 5 times a power of ten).
///
/// With `round`, snaps to the nearest nice number; otherwise rounds up so the
/// result covers the span. Used to keep axis ticks on human-friendly values.
fn nice_num(span: f64, round: bool) -> f64 {
    if span <= 0.0 {
        return 1.0;
    }

    let exp = span.log10().floor();
    let frac = span / 10f64.powf(exp);
    let nice = if round {
        if frac < 1.5 {
            1.0
        } else if frac < 3.0 {
            2.0
        } else if frac < 7.0 {
            5.0
        } else {
            10.0
        }
    } else if frac <= 1.0 {
        1.0
    } else if frac <= 2.0 {
        2.0
    } else if frac <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice * 10f64.powf(exp)
}

/// Picks roughly `target` evenly-spaced, round-numbered price ticks spanning
/// `[min, max]`. Ticks outside the domain are dropped by the caller as needed.
pub(crate) fn price_ticks(min: f64, max: f64, target: usize) -> Vec<f64> {
    let target = target.max(2);
    let range = nice_num(max - min, false);
    let step = nice_num(range / (target as f64 - 1.0), true);

    if step <= 0.0 || !step.is_finite() {
        return vec![min, max];
    }

    let graph_min = (min / step).floor() * step;
    let graph_max = (max / step).ceil() * step;
    let mut ticks = Vec::new();
    let mut t = graph_min;

    // Guard against runaway loops from pathological inputs.
    let max_ticks = target * 4;
    while t <= graph_max + step * 0.5 && ticks.len() < max_ticks {
        ticks.push(t);
        t += step;
    }

    ticks
}

/// Formats a price for an axis label, choosing decimal places from the tick
/// spacing so small ranges keep precision and large ones stay compact.
pub(crate) fn format_price(value: f64, step: f64) -> String {
    let decimals = if step >= 1.0 {
        0
    } else if step >= 0.1 {
        1
    } else if step >= 0.01 {
        2
    } else {
        3
    };

    format!("{value:.decimals$}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_are_round_and_evenly_spaced() {
        let ticks = price_ticks(0.0, 100.0, 6);
        assert!(ticks.len() >= 2);
        let step = ticks[1] - ticks[0];
        // The step is a nice number, and spacing is uniform.
        assert!((step - 20.0).abs() < 1e-9, "unexpected step {step}");
        for pair in ticks.windows(2) {
            assert!((pair[1] - pair[0] - step).abs() < 1e-9);
        }
    }

    #[test]
    fn ticks_bracket_the_domain() {
        let ticks = price_ticks(13.0, 87.0, 5);
        assert!(ticks.first().unwrap() <= &13.0);
        assert!(ticks.last().unwrap() >= &87.0);
    }

    #[test]
    fn format_price_picks_decimals_from_step() {
        assert_eq!(format_price(123.456, 5.0), "123");
        assert_eq!(format_price(123.456, 0.5), "123.5");
        assert_eq!(format_price(1.2345, 0.05), "1.23");
        assert_eq!(format_price(1.2345, 0.005), "1.234");
    }

    #[test]
    fn degenerate_span_does_not_panic() {
        let ticks = price_ticks(50.0, 50.0, 6);
        assert!(!ticks.is_empty());
        assert!(ticks.iter().all(|t| t.is_finite()));
    }

    #[test]
    fn price_axis_defaults_and_builders() {
        let axis = PriceAxis::default();
        assert_eq!(axis.width, 8);
        assert_eq!(axis.style.fg, Some(Color::Gray));

        let axis = axis.width(10).style(Color::Red);
        assert_eq!(axis.width, 10);
        assert_eq!(axis.style.fg, Some(Color::Red));
    }

    #[test]
    fn time_axis_defaults_and_builders() {
        let axis = TimeAxis::default();
        assert_eq!(axis.style.fg, Some(Color::Gray));
        assert!(axis.labels.is_none());

        let labels = [String::from("a"), String::from("b")];
        let axis = axis.labels(&labels).style(Color::Blue);
        assert_eq!(axis.labels.unwrap().len(), 2);
        assert_eq!(axis.style.fg, Some(Color::Blue));
    }
}
