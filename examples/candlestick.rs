//! A complete terminal app that renders a candlestick chart from static data.
//!
//! Run it with:
//!
//! ```sh
//! cargo run --example candlestick
//! ```
//!
//! Press `q` or `Esc` to quit.

use chandelier::{Candle, CandleSeries, CandlestickChart, PriceAxis, TimeAxis};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};

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
    let area = frame.area();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" chandelier: ACME daily (q to quit) ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // The candle series owns the data and its bull/bear/wick colors. A neutral
    // wick color, distinct from the bodies, keeps wicks legible against either.
    let series = CandleSeries::new(candles)
        .width(1.0)
        .gap(1.0)
        .bull_style(Color::Rgb(38, 166, 154))
        .bear_style(Color::Rgb(239, 83, 80))
        .wick_style(Color::Rgb(110, 116, 130));

    // A dark chart background here is purely cosmetic. Partial bodies render
    // correctly over the terminal default too, so the background is optional.
    let axis_style = Style::new().fg(Color::Rgb(120, 123, 134));
    let chart = CandlestickChart::new(series)
        .style(Style::new().bg(Color::Rgb(13, 17, 23)))
        .price_axis(PriceAxis::default().style(axis_style))
        .time_axis(TimeAxis::default().style(axis_style).labels(labels));

    frame.render_widget(&chart, inner);
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

/// Sequential "day" labels for the x-axis.
fn sample_labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("d{i}")).collect()
}
