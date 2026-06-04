//! A terminal app that plots indicator [`LineChart`]s: an RSI pane with
//! overbought/oversold [`TrendLine`] reference levels, and a MACD pane with two
//! lines (the MACD line and its signal) sharing one value axis.
//!
//! Run it with:
//!
//! ```sh
//! cargo run --example line
//! ```
//!
//! Press `q` or `Esc` to quit.

use chandelier::{Candle, Label, LineChart, LineSeries, TimeAxis, TrendLine, ValueAxis};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ta::Next;
use ta::indicators::{MovingAverageConvergenceDivergence, RelativeStrengthIndex};

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
    let up = Color::Rgb(38, 166, 154);
    let down = Color::Rgb(239, 83, 80);
    let axis_style = Style::new().fg(Color::Rgb(120, 123, 134));
    let base = Style::new().bg(Color::Rgb(13, 17, 23));

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    // Stack the two panes, RSI above MACD. Each chart lays its lines out the
    // same way a candle series would, so they line up column-for-column.
    let [top, bottom] = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(frame.area());

    // RSI: a single line on a value axis pinned to its natural 0..100 range, so
    // the dashed reference levels at the conventional overbought (70) and
    // oversold (30) thresholds stay fixed instead of drifting with autoscale.
    let rsi_values = rsi(&closes, 14);
    let rsi_chart = LineChart::new(LineSeries::new(&rsi_values).style(Color::Rgb(124, 179, 66)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" chandelier: RSI (14) (q to quit) "),
        )
        .style(base)
        .width(1.0)
        .gap(1.0)
        .value_axis(
            ValueAxis::default()
                .style(axis_style)
                .bounds([0.0, 100.0])
                .ticks(&[0.0, 30.0, 70.0, 100.0]),
        )
        .time_axis(TimeAxis::default().style(axis_style).labels(labels))
        .overlay(
            TrendLine::at(70.0)
                .dashed()
                .style(down)
                .label(Label::new("OVERBOUGHT").alignment(Alignment::Left).inset(2)),
        )
        .overlay(
            TrendLine::at(30.0)
                .dashed()
                .style(up)
                .label(Label::new("OVERSOLD").alignment(Alignment::Left).inset(2)),
        );

    // MACD: two equal-status lines, the MACD line and its signal, sharing one
    // autoscaling axis, with a zero line drawn behind them.
    let (macd_line, signal_line) = macd(&closes);
    let macd_chart = LineChart::new(LineSeries::new(&macd_line).style(Color::Rgb(41, 182, 246)))
        .line(LineSeries::new(&signal_line).style(Color::Rgb(255, 167, 38)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" MACD (12, 26, 9) "),
        )
        .style(base)
        .width(1.0)
        .gap(1.0)
        .value_axis(ValueAxis::default().style(axis_style))
        .time_axis(TimeAxis::default().style(axis_style).labels(labels))
        .underlay(TrendLine::at(0.0).style(axis_style));

    frame.render_widget(&rsi_chart, top);
    frame.render_widget(&macd_chart, bottom);
}

/// A synthetic price series. The generator is seeded so the example renders the
/// same chart on every run.
fn sample_candles() -> Vec<Candle> {
    let mut rng = StdRng::seed_from_u64(0x9E37_79B9_7F4A_7C15);

    let mut candles = Vec::with_capacity(96);
    let mut price = 100.0_f64;
    for _ in 0..96 {
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

/// The RSI of `closes` over `period`, aligned one-to-one with the closes,
/// computed with the [`ta`] crate.
fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut rsi = RelativeStrengthIndex::new(period).expect("period is non-zero");
    closes.iter().map(|&close| Some(rsi.next(close))).collect()
}

/// The MACD line and its signal line, aligned one-to-one with `closes`, computed
/// with the [`ta`] crate from the conventional 12, 26, and 9 period settings.
fn macd(closes: &[f64]) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).expect("periods are valid");
    closes
        .iter()
        .map(|&close| {
            let out = macd.next(close);
            (Some(out.macd), Some(out.signal))
        })
        .unzip()
}

/// Sequential "day" labels for the x-axis.
fn sample_labels(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("d{i}")).collect()
}
