//! Tick selection for the price and time axes.
//!
//! These are pure helpers. They pick where labels go and how they read; the
//! chart turns the positions into buffer writes.

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
pub fn price_ticks(min: f64, max: f64, target: usize) -> Vec<f64> {
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
pub fn format_price(value: f64, step: f64) -> String {
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
}
