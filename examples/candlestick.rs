//! A complete terminal app that renders a candlestick chart with a volume chart
//! stacked beneath it, from static data.
//!
//! Run it with:
//!
//! ```sh
//! cargo run --example candlestick
//! ```
//!
//! Press `q` or `Esc` to quit.

use chandelier::{
    Candle, CandleSeries, CandlestickChart, PriceAxis, TimeAxis, ValueAxis, Volume, VolumeChart,
    VolumeSeries,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};

fn main() -> std::io::Result<()> {
    let candles = sample_candles();
    let volumes = sample_volumes(&candles);
    let labels = sample_labels(candles.len());

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &candles, &volumes, &labels);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    candles: &[Candle],
    volumes: &[Volume],
    labels: &[String],
) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, candles, volumes, labels))?;
        if let Event::Key(key) = event::read()?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(());
        }
    }
}

fn draw(frame: &mut Frame, candles: &[Candle], volumes: &[Volume], labels: &[String]) {
    // Stack the price chart over a shorter volume chart.
    let [price_area, volume_area] =
        Layout::vertical([Constraint::Percentage(67), Constraint::Percentage(33)])
            .areas(frame.area());

    let bull = Color::Rgb(38, 166, 154);
    let bear = Color::Rgb(239, 83, 80);
    let axis_style = Style::new().fg(Color::Rgb(120, 123, 134));
    let base = Style::new().bg(Color::Rgb(13, 17, 23));

    // The candle series defines the data and sets rendering and style options.
    let candle_series = CandleSeries::new(candles)
        .width(1.0)
        .gap(1.0)
        .bull_style(bull)
        .bear_style(bear)
        .wick_style(Color::Rgb(110, 116, 130));
    let price_chart = CandlestickChart::new(candle_series)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" chandelier: ACME daily (q to quit) "),
        )
        .style(base)
        .price_axis(PriceAxis::default().style(axis_style))
        .time_axis(TimeAxis::default().style(axis_style).labels(labels));

    // The volume series shares the candles' width, gap, and length, and both
    // charts reserve the same axis width, so their time axes line up.
    let volume_series = VolumeSeries::new(volumes)
        .width(1.0)
        .gap(1.0)
        .bull_style(bull)
        .bear_style(bear);
    let volume_chart = VolumeChart::new(volume_series)
        .block(Block::default().borders(Borders::ALL).title(" Volume "))
        .style(base)
        .value_axis(ValueAxis::default().style(axis_style))
        .time_axis(TimeAxis::default().style(axis_style).labels(labels));

    frame.render_widget(&price_chart, price_area);
    frame.render_widget(&volume_chart, volume_area);
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

/// Synthetic volumes, one per candle.
fn sample_volumes(candles: &[Candle]) -> Vec<Volume> {
    let mut rng = StdRng::seed_from_u64(0x1234_5678_9ABC_DEF0);

    candles
        .iter()
        .map(|candle| {
            let move_size = (candle.close - candle.open).abs();
            let quantity = 500.0 + move_size * 400.0 + rng.random_range(0.0..1.0) * 800.0;
            Volume::new(quantity).with_direction(candle.direction())
        })
        .collect()
}

/// Sequential "day" labels for the x-axis.
fn sample_labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("d{i}")).collect()
}
