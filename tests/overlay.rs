//! Renders charts with overlays into an in-memory buffer and asserts what lands
//! on the grid.

use chandelier::{
    Candle, CandleSeries, CandlestickChart, LineStyle, ValueLine, Volume, VolumeChart, VolumeSeries,
};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Rect};
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::widgets::Widget;

fn render(chart: &CandlestickChart, w: u16, h: u16) -> Buffer {
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    chart.render(area, &mut buf);
    buf
}

/// The whole row `y` as a string of its cell symbols.
fn row_text(buf: &Buffer, y: u16) -> String {
    let area = *buf.area();
    (area.x..area.x + area.width)
        .map(|x| buf[(x, y)].symbol())
        .collect()
}

/// The first row carrying the given symbol anywhere, if any.
fn row_with_symbol(buf: &Buffer, symbol: &str) -> Option<u16> {
    let area = *buf.area();
    (area.y..area.y + area.height)
        .find(|&y| (area.x..area.x + area.width).any(|x| buf[(x, y)].symbol() == symbol))
}

fn candles() -> [Candle; 3] {
    [
        Candle::new(100.0, 105.0, 99.0, 104.0),
        Candle::new(104.0, 108.0, 103.0, 106.0),
        Candle::new(106.0, 107.0, 101.0, 102.0),
    ]
}

#[test]
fn value_line_spans_the_whole_plot_row() {
    let candles = candles();
    let line = ValueLine::at(104.0).style(Color::Yellow);
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(line);
    let buf = render(&chart, 24, 12);

    let y = row_with_symbol(&buf, "─").expect("expected a solid line row");
    // Every column of that row carries the line glyph in the line color.
    for x in 0..24 {
        assert_eq!(buf[(x, y)].symbol(), "─", "gap at column {x}");
        assert_eq!(buf[(x, y)].fg, Color::Yellow);
    }
}

#[test]
fn value_line_sits_at_the_value_row() {
    // A line at the very top of the candle range lands high; one near the bottom
    // lands low. With axes off the plot is the full area.
    let candles = candles();
    let high = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .padding(0.0)
        .overlay(ValueLine::at(108.0));
    let low = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .padding(0.0)
        .overlay(ValueLine::at(99.0));

    let high_row = row_with_symbol(&render(&high, 24, 12), "─").unwrap();
    let low_row = row_with_symbol(&render(&low, 24, 12), "─").unwrap();
    assert!(
        high_row < low_row,
        "higher value should be on a smaller row"
    );
}

#[test]
fn dashed_and_solid_use_distinct_glyphs() {
    let candles = candles();
    let solid = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(ValueLine::at(104.0));
    let dashed = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(ValueLine::at(104.0).line(LineStyle::Dashed));

    assert!(row_with_symbol(&render(&solid, 24, 12), "─").is_some());
    assert!(row_with_symbol(&render(&solid, 24, 12), "╌").is_none());
    assert!(row_with_symbol(&render(&dashed, 24, 12), "╌").is_some());
}

#[test]
fn label_is_right_aligned_by_default() {
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(ValueLine::at(104.0).label("LAST"));
    let buf = render(&chart, 24, 12);

    let y = row_with_symbol(&buf, "─").unwrap();
    let text = row_text(&buf, y);
    assert!(text.ends_with("LAST"), "label not right-aligned: {text:?}");
}

#[test]
fn label_left_alignment_starts_at_the_left_edge() {
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(
            ValueLine::at(104.0)
                .label("LAST")
                .label_alignment(Alignment::Left),
        );
    let buf = render(&chart, 24, 12);

    let y = row_with_symbol(&buf, "─").unwrap();
    let text = row_text(&buf, y);
    assert!(text.starts_with("LAST"), "label not left-aligned: {text:?}");
}

#[test]
fn centered_label_breaks_the_line_with_padding() {
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(
            ValueLine::at(104.0)
                .label("LAST")
                .label_alignment(Alignment::Center)
                .label_padding(2),
        );
    let buf = render(&chart, 24, 12);

    let y = row_with_symbol(&buf, "─").unwrap();
    let text = row_text(&buf, y);
    // The line reaches both edges, with a padded gap around the centered label.
    assert!(
        text.starts_with("─"),
        "line should reach the left edge: {text:?}"
    );
    assert!(
        text.ends_with("─"),
        "line should reach the right edge: {text:?}"
    );
    assert!(
        text.contains("  LAST  "),
        "label not padded by blanks: {text:?}"
    );
}

#[test]
fn label_inset_leads_in_with_line_before_the_label() {
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(
            ValueLine::at(104.0)
                .label("RES")
                .label_alignment(Alignment::Left)
                .label_inset(2)
                .label_padding(0),
        );
    let buf = render(&chart, 24, 12);

    let y = row_with_symbol(&buf, "─").unwrap();
    let text = row_text(&buf, y);
    // Two leading line columns, then the label, then the line continues.
    assert!(
        text.starts_with("──RES─"),
        "expected an inset label: {text:?}"
    );
}

#[test]
fn line_and_label_paint_on_the_chart_background() {
    // The line and its label sit on the chart background, never inheriting a
    // candle cell's REVERSED inversion (which would swap the colors).
    let base = Color::Rgb(13, 17, 23);
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .style(Style::new().bg(base))
        .overlay(
            ValueLine::at(104.0)
                .style(Color::White)
                .label("LAST")
                .label_alignment(Alignment::Center),
        );
    let buf = render(&chart, 24, 12);

    let y = row_with_symbol(&buf, "─").unwrap();
    for x in 0..24 {
        let cell = &buf[(x, y)];
        assert_eq!(cell.bg, base, "cell {x} should sit on the chart background");
        assert!(
            !cell.modifier.contains(Modifier::REVERSED),
            "cell {x} should not be reversed"
        );
    }
}

#[test]
fn overlay_only_touches_its_own_row() {
    let candles = candles();
    let plain = CandlestickChart::new(CandleSeries::new(&candles)).axes(false);
    let with_line = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(ValueLine::at(104.0));

    let before = render(&plain, 24, 12);
    let after = render(&with_line, 24, 12);
    let y = row_with_symbol(&after, "─").unwrap();

    // Every row except the line's is unchanged by the overlay.
    for row in 0..12 {
        if row == y {
            continue;
        }
        assert_eq!(
            row_text(&before, row),
            row_text(&after, row),
            "row {row} changed"
        );
    }
}

#[test]
fn autoscale_expands_to_keep_an_out_of_range_line_visible() {
    // A line far above every candle high is still drawn when autoscale is on,
    // and the price axis grows to include it.
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles)).overlay(ValueLine::at(150.0));
    let buf = render(&chart, 40, 16);

    assert!(
        row_with_symbol(&buf, "─").is_some(),
        "line should be visible"
    );
    // A round-number tick above every candle high (108) only appears if the
    // axis grew toward the line at 150.
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(
        text.contains("120"),
        "price axis should expand past the candles toward the line"
    );
}

#[test]
fn autoscale_off_pins_the_axis_and_clamps_the_line() {
    // With autoscale off the axis ignores the line, which clamps to the top row.
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .overlay(ValueLine::at(150.0).autoscale(false));
    let buf = render(&chart, 40, 16);

    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(!text.contains("150"), "axis should not expand to the line");
    // The clamped line sits on the very top row of the plot.
    assert_eq!(row_with_symbol(&buf, "─"), Some(0));
}

#[test]
fn volume_overlay_raises_the_top_but_keeps_the_zero_floor() {
    // A threshold above the tallest bar lifts the top of the value axis; the
    // baseline stays at zero so the bars still sit on the floor.
    let volumes = [Volume::new(10.0), Volume::new(20.0), Volume::new(15.0)];
    let area = Rect::new(0, 0, 30, 12);

    let plain = VolumeChart::new(VolumeSeries::new(&volumes));
    let mut plain_buf = Buffer::empty(area);
    plain.render(area, &mut plain_buf);

    let chart = VolumeChart::new(VolumeSeries::new(&volumes))
        .overlay(ValueLine::at(40.0).style(Style::new().fg(Color::Cyan)));
    let mut buf = Buffer::empty(area);
    chart.render(area, &mut buf);

    // The line is drawn above the bars, in the top half of the plot.
    let line_row = (0..12)
        .find(|&y| (0..30).any(|x| buf[(x, y)].symbol() == "─"))
        .expect("threshold line should be visible");
    assert!(line_row < 6, "threshold should sit high above the bars");

    // The bottom row still holds bars in both, so the floor did not move.
    let floor = 12 - 1 - 1; // minus the time axis row
    assert_eq!(row_text(&plain_buf, floor), row_text(&buf, floor));
}
