//! Renders charts into an in-memory buffer and asserts what lands on the grid.
//! This verifies the renderer without an interactive terminal.

use chandelier::{Candle, CandleSeries, CandlestickChart, PriceAxis, TimeAxis};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::widgets::Widget;

/// Collects the foreground colors actually used across the buffer.
fn foreground_colors(buf: &Buffer) -> Vec<Color> {
    let mut seen = Vec::new();
    for cell in buf.content() {
        if cell.symbol() != " " && !seen.contains(&cell.fg) {
            seen.push(cell.fg);
        }
    }
    seen
}

fn render(chart: &CandlestickChart, w: u16, h: u16) -> Buffer {
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    chart.render(area, &mut buf);
    buf
}

#[test]
fn empty_series_renders_nothing() {
    let chart = CandlestickChart::new(CandleSeries::new(&[]));
    let buf = render(&chart, 20, 10);
    assert!(buf.content().iter().all(|c| c.symbol() == " "));
}

#[test]
fn bull_and_bear_use_distinct_colors() {
    let candles = [
        Candle::new(100.0, 110.0, 98.0, 108.0), // bull
        Candle::new(108.0, 109.0, 95.0, 96.0),  // bear
    ];
    let series = CandleSeries::new(&candles)
        .width(3)
        .bull_style(Color::Green)
        .bear_style(Color::Red);
    let chart = CandlestickChart::new(series)
        .style(Style::new().bg(Color::Black))
        .axes(false);
    let buf = render(&chart, 30, 16);

    let fg = foreground_colors(&buf);
    assert!(fg.contains(&Color::Green), "expected a green (bull) candle");
    assert!(fg.contains(&Color::Red), "expected a red (bear) candle");
}

#[test]
fn draws_wick_and_body_glyphs() {
    let candles = [Candle::new(50.0, 60.0, 40.0, 55.0)];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3)).axes(false);
    let buf = render(&chart, 20, 16);

    let symbols: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(symbols.contains('│'), "expected a wick glyph");
    assert!(
        symbols.contains('█') || symbols.chars().any(|c| ('▁'..='▇').contains(&c)),
        "expected a full or partial body block"
    );
}

#[test]
fn price_axis_labels_are_drawn() {
    let candles = [
        Candle::new(100.0, 105.0, 99.0, 104.0),
        Candle::new(104.0, 108.0, 103.0, 106.0),
    ];
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .price_axis(PriceAxis::default().width(8));
    let buf = render(&chart, 40, 14);

    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(
        text.chars().any(|c| c.is_ascii_digit()),
        "expected numeric price-axis labels"
    );
}

#[test]
fn renders_within_bounds_for_many_candles() {
    // A series far larger than the area must not panic and must clip cleanly.
    let candles: Vec<Candle> = (0..500)
        .map(|i| {
            let base = 100.0 + (i as f64 % 17.0);
            Candle::new(base, base + 3.0, base - 2.0, base + 1.0)
        })
        .collect();
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(1).gap(0));
    let _ = render(&chart, 24, 12); // must not panic
}

const PARTIAL_BLOCKS: [char; 7] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];

fn is_partial_block(symbol: &str) -> bool {
    symbol.chars().any(|c| PARTIAL_BLOCKS.contains(&c))
}

#[test]
fn partial_body_edges_keep_the_background_and_never_overstate_the_body() {
    // A partial edge cell must keep the background in its empty half, never the
    // body color, which would render the open/close higher or lower than it
    // really is. Wicks float free of the body, so they never fill that half.
    let bg = Color::Rgb(10, 10, 12);
    let bull = Color::Rgb(0, 200, 120);
    let wick = Color::Rgb(110, 116, 130);
    let candles = [Candle::new(100.0, 130.0, 70.0, 104.0)];
    let series = CandleSeries::new(&candles)
        .width(1)
        .bull_style(bull)
        .wick_style(wick);
    let chart = CandlestickChart::new(series)
        .style(Style::new().bg(bg))
        .axes(false);
    let buf = render(&chart, 1, 24);

    for y in 0..buf.area.height {
        let cell = &buf[(0, y)];
        if is_partial_block(cell.symbol()) {
            let colors = [cell.fg, cell.bg];
            assert!(
                colors.iter().all(|c| *c == bull || *c == bg),
                "partial edge cell at row {y} should be body-on-background, got {colors:?}"
            );
        }
    }
}

#[test]
fn partial_wick_tips_use_a_half_glyph_in_the_wick_color() {
    // A high or low that lands mid-cell is drawn with a half-height vertical tip
    // glyph (╵ or ╷) rather than snapping to a whole row, painted in the wick
    // color over the background. (At this height the low lands on a half-row.)
    let bg = Color::Rgb(10, 10, 12);
    let bull = Color::Rgb(0, 200, 120);
    let wick = Color::Rgb(110, 116, 130);
    let candles = [Candle::new(100.0, 130.0, 70.0, 104.0)];
    let series = CandleSeries::new(&candles)
        .width(1)
        .bull_style(bull)
        .wick_style(wick);
    let chart = CandlestickChart::new(series)
        .style(Style::new().bg(bg))
        .axes(false);
    let buf = render(&chart, 1, 24);

    let mut saw_half_tip = false;
    for y in 0..buf.area.height {
        let cell = &buf[(0, y)];
        if cell.symbol() == "╵" || cell.symbol() == "╷" {
            saw_half_tip = true;
            assert_eq!(
                cell.fg, wick,
                "half wick tip at row {y} should be the wick color"
            );
            assert_eq!(
                cell.bg, bg,
                "half wick tip at row {y} should sit on the background"
            );
        }
    }
    assert!(saw_half_tip, "expected a half-row wick tip");
}

#[test]
fn partial_bodies_render_over_a_transparent_background() {
    // With no chart background set, partial body cells are the body color in the
    // foreground over a Reset (terminal default) background, so the chart needs
    // no background to render correctly.
    let bull = Color::Rgb(0, 200, 120);
    let candles = [Candle::new(100.0, 130.0, 70.0, 104.0)];
    let series = CandleSeries::new(&candles).width(1).bull_style(bull);
    let chart = CandlestickChart::new(series).axes(false); // no .style(): bg stays Reset
    let buf = render(&chart, 1, 24);

    let mut saw_partial = false;
    for y in 0..buf.area.height {
        let cell = &buf[(0, y)];
        if is_partial_block(cell.symbol()) {
            saw_partial = true;
            assert_eq!(
                cell.fg, bull,
                "partial body at row {y} should be body-colored"
            );
            assert_eq!(
                cell.bg,
                Color::Reset,
                "partial body at row {y} should sit on the terminal default background"
            );
        }
    }
    assert!(saw_partial, "expected at least one partial body cell");
}

#[test]
fn candle_glyph_grid_is_stable() {
    // Small test chart to prevent rendering regression.
    let candles = [
        Candle::new(100.0, 106.0, 99.0, 105.0),
        Candle::new(105.0, 109.0, 104.0, 104.5),
        Candle::new(104.5, 105.0, 98.0, 99.0),
    ];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3).gap(1)).axes(false);
    let buf = render(&chart, 12, 8);

    let mut grid: Vec<String> = Vec::new();
    for y in 0..buf.area.height {
        let mut row = String::new();
        for x in 0..buf.area.width {
            row.push_str(buf[(x, y)].symbol());
        }
        grid.push(row);
    }

    let expected = [
        "     ╷      ",
        "     │      ",
        " ╷   │      ",
        "███ ▅▅▅ ▅▅▅ ",
        "███     ███ ",
        "███     ███ ",
        "▅▅▅     ███ ",
        " ╵       │  ",
    ];

    assert_eq!(grid, expected);
}

/// The truecolor SGR prefix for a cell's foreground and background, so the dump
/// reflects fg/bg-inverted cells (a body lit from the top is inverted).
fn sgr(fg: Color, bg: Color) -> String {
    let mut codes = Vec::new();
    if let Color::Rgb(r, g, b) = fg {
        codes.push(format!("38;2;{r};{g};{b}"));
    }
    if let Color::Rgb(r, g, b) = bg {
        codes.push(format!("48;2;{r};{g};{b}"));
    }
    if codes.is_empty() {
        String::new()
    } else {
        format!("\x1b[{}m", codes.join(";"))
    }
}

/// Dumps a small chart to stdout in color so it can be eyeballed with
/// `cargo test --test render show_chart -- --nocapture` (needs a truecolor
/// terminal). Inverted characters are not rendered correctly without color.
#[test]
fn show_chart() {
    let candles = [
        Candle::new(100.0, 106.0, 99.0, 105.0),
        Candle::new(105.0, 109.0, 104.0, 104.5),
        Candle::new(104.5, 105.0, 98.0, 99.0),
        Candle::new(99.0, 103.0, 97.0, 102.0),
        Candle::new(102.0, 108.0, 101.0, 107.5),
        Candle::new(107.5, 110.0, 106.0, 106.5),
    ];
    let series = CandleSeries::new(&candles)
        .width(3)
        .gap(1)
        .bull_style(Color::Rgb(38, 166, 154))
        .bear_style(Color::Rgb(239, 83, 80))
        .wick_style(Color::Rgb(110, 116, 130));
    let axis_style = Color::Rgb(120, 123, 134);
    let chart = CandlestickChart::new(series)
        .style(Style::new().bg(Color::Rgb(13, 17, 23)))
        .price_axis(PriceAxis::default().style(axis_style))
        .time_axis(TimeAxis::default().style(axis_style));
    let buf = render(&chart, 44, 16);

    println!();
    for y in 0..buf.area.height {
        let mut line = String::new();
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            // Reset per cell, then set colors and reverse, so a reversed cell
            // (a body lit from the top) is shown the way the terminal draws it.
            line.push_str("\x1b[0m");
            line.push_str(&sgr(cell.fg, cell.bg));
            if cell.modifier.contains(Modifier::REVERSED) {
                line.push_str("\x1b[7m");
            }
            line.push_str(cell.symbol());
        }
        line.push_str("\x1b[0m");
        println!("{line}");
    }
}
