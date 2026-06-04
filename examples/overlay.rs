//! A terminal app that layers overlays on a candlestick chart: a moving-average
//! [`LineOverlay`], horizontal [`TrendLine`] reference levels with differently
//! aligned labels, and buy/sell [`Annotation`] markers.
//!
//! Run it with:
//!
//! ```sh
//! cargo run --example overlay
//! ```
//!
//! Press `q` or `Esc` to quit.

use chandelier::{
    Annotation, Annotations, Candle, CandleSeries, CandlestickChart, Label, LineOverlay, PriceAxis,
    TimeAxis, TrendLine, price_bounds,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ta::Next;
use ta::indicators::SimpleMovingAverage;

fn main() -> std::io::Result<()> {
    let candles = sample_candles();
    let labels = sample_labels(candles.len());

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &candles, &labels);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    candles: &[Candle],
    labels: &[String],
) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, candles, labels))?;
        if let Event::Key(key) = event::read()?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(());
        }
    }
}

fn draw(frame: &mut Frame, candles: &[Candle], labels: &[String]) {
    let bull = Color::Rgb(38, 166, 154);
    let bear = Color::Rgb(239, 83, 80);
    let axis_style = Style::new().fg(Color::Rgb(120, 123, 134));
    let base = Style::new().bg(Color::Rgb(13, 17, 23));

    // Reference levels: the latest close, the highest high as resistance, and a
    // fixed support level.
    let last = candles.last().map_or(0.0, |c| c.close);
    let (_, high) = price_bounds(candles).unwrap_or((0.0, 0.0));

    // Add buy/sell markers at a candle's low and high.
    let n = candles.len();
    let trades = [
        Annotation::buy(n - 12, candles[n - 12].low),
        Annotation::sell(n - 5, candles[n - 5].high),
    ];

    // A 10-period simple moving average of the closes, tracking the candles.
    let moving_average = sma(candles, 10);

    let candle_series = CandleSeries::new(candles)
        .width(1.0)
        .gap(1.0)
        .bull_style(bull)
        .bear_style(bear)
        .wick_style(Color::Rgb(110, 116, 130));
    let chart = CandlestickChart::new(candle_series)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" chandelier: trend lines (q to quit) "),
        )
        .style(base)
        .price_axis(PriceAxis::default().style(axis_style))
        .time_axis(TimeAxis::default().style(axis_style).labels(labels))
        // A moving average tracking the candles, drawn behind them so the
        // candles sit on top of it.
        .underlay(LineOverlay::new(&moving_average).style(Color::Rgb(255, 202, 40)))
        // The last close: a solid line, label against the price axis (default).
        .overlay(
            TrendLine::at(last)
                .style(Color::Rgb(230, 230, 230))
                .label("LAST"),
        )
        // Resistance: a dashed line at the highest high, label inset from the
        // left so a short run of line leads into it.
        .overlay(
            TrendLine::at(high)
                .dashed()
                .style(bear)
                .label(Label::new("RESISTANCE").alignment(Alignment::Left).inset(2)),
        )
        // Support: a dashed line at a fixed level, label centered so the line
        // breaks around it on both sides.
        .overlay(
            TrendLine::at(96.0)
                .dashed()
                .style(bull)
                .label(Label::new("SUPPORT").alignment(Alignment::Center)),
        )
        // Buy/sell markers, drawn with their conventional triangles and labels.
        .overlay(Annotations::new(&trades));

    frame.render_widget(&chart, frame.area());
}

/// A synthetic price series. The generator is seeded so the example renders the
/// same chart on every run.
fn sample_candles() -> Vec<Candle> {
    let mut rng = StdRng::seed_from_u64(0x9E37_79B9_7F4A_7C15);

    let mut candles = Vec::with_capacity(72);
    let mut price = 100.0_f64;
    for _ in 0..72 {
        let drift = (rng.random_range(0.0..1.0) - 0.48) * 4.0;
        let open = price;
        let close = (open + drift).max(1.0);
        let span = 0.5 + rng.random_range(0.0..1.0) * 2.5;
        let high = open.max(close) + rng.random_range(0.0..1.0) * span;
        let low = open.min(close) - rng.random_range(0.0..1.0) * span;
        candles.push(Candle::new(open, high, low.max(0.5), close));
        price = close;
    }
    candles
}

/// A simple moving average of the candle closes over `period` candles, aligned
/// one-to-one with the candles, computed with the [`ta`] crate.
fn sma(candles: &[Candle], period: usize) -> Vec<Option<f64>> {
    let mut sma = SimpleMovingAverage::new(period).expect("period is non-zero");
    candles
        .iter()
        .map(|candle| Some(sma.next(candle.close)))
        .collect()
}

/// Sequential "day" labels for the x-axis.
fn sample_labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("d{i}")).collect()
}
