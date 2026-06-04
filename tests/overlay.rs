//! Renders charts with overlays into an in-memory buffer and asserts what lands
//! on the grid.

use chandelier::{
    Anchor, Annotation, Annotations, Candle, CandleSeries, CandlestickChart, Label, LineOverlay,
    LineStyle, TrendLine, Volume, VolumeChart, VolumeSeries,
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

/// The coordinates of the first cell whose symbol equals `symbol`.
fn cell_of(buf: &Buffer, symbol: &str) -> Option<(u16, u16)> {
    let area = *buf.area();
    (area.y..area.y + area.height)
        .flat_map(|y| (area.x..area.x + area.width).map(move |x| (x, y)))
        .find(|&(x, y)| buf[(x, y)].symbol() == symbol)
}

/// How many painted cells carry the given foreground color.
fn count_fg(buf: &Buffer, color: Color) -> usize {
    buf.content()
        .iter()
        .filter(|c| c.symbol() != " " && c.fg == color)
        .count()
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
    let line = TrendLine::at(104.0).style(Color::Yellow);
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
        .overlay(TrendLine::at(108.0));
    let low = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .padding(0.0)
        .overlay(TrendLine::at(99.0));

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
        .overlay(TrendLine::at(104.0));
    let dashed = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(TrendLine::at(104.0).line(LineStyle::Dashed));

    assert!(row_with_symbol(&render(&solid, 24, 12), "─").is_some());
    assert!(row_with_symbol(&render(&solid, 24, 12), "╌").is_none());
    assert!(row_with_symbol(&render(&dashed, 24, 12), "╌").is_some());
}

#[test]
fn label_is_right_aligned_by_default() {
    let candles = candles();
    let chart = CandlestickChart::new(CandleSeries::new(&candles))
        .axes(false)
        .overlay(TrendLine::at(104.0).label("LAST"));
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
        .overlay(TrendLine::at(104.0).label(Label::new("LAST").alignment(Alignment::Left)));
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
            TrendLine::at(104.0).label(Label::new("LAST").alignment(Alignment::Center).padding(2)),
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
            TrendLine::at(104.0).label(
                Label::new("RES")
                    .alignment(Alignment::Left)
                    .inset(2)
                    .padding(0),
            ),
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
            TrendLine::at(104.0)
                .style(Color::White)
                .label(Label::new("LAST").alignment(Alignment::Center)),
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
        .overlay(TrendLine::at(104.0));

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
    let chart = CandlestickChart::new(CandleSeries::new(&candles)).overlay(TrendLine::at(150.0));
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
        .overlay(TrendLine::at(150.0).autoscale(false));
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
        .overlay(TrendLine::at(40.0).style(Style::new().fg(Color::Cyan)));
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

#[test]
fn line_overlay_paints_in_its_color() {
    let candles = candles();
    let values = [Some(104.0), Some(104.0), Some(104.0)];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .overlay(LineOverlay::new(&values).style(Color::Magenta));
    let buf = render(&chart, 24, 12);

    assert!(
        count_fg(&buf, Color::Magenta) > 0,
        "line color should appear"
    );
}

#[test]
fn a_gap_breaks_the_line() {
    let candles = candles();
    let full = [Some(101.0), Some(106.0), Some(102.0)];
    let gapped = [Some(101.0), None, Some(102.0)];
    let painted = |values: &[Option<f64>]| {
        let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
            .axes(false)
            .overlay(LineOverlay::new(values).style(Color::Magenta));
        count_fg(&render(&chart, 24, 12), Color::Magenta)
    };

    assert!(
        painted(&gapped) < painted(&full),
        "a None should drop the line cells it would have connected"
    );
}

#[test]
fn an_underlay_is_occluded_by_the_candles() {
    let candles = candles();
    let values = [Some(104.0), Some(104.0), Some(104.0)];
    let front = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .overlay(LineOverlay::new(&values).style(Color::Magenta));
    let behind = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .underlay(LineOverlay::new(&values).style(Color::Magenta));

    assert!(
        count_fg(&render(&behind, 24, 12), Color::Magenta)
            < count_fg(&render(&front, 24, 12), Color::Magenta),
        "candles drawn after an underlay should occlude part of it"
    );
}

#[test]
fn all_none_line_draws_nothing() {
    let candles = candles();
    let none = [None, None, None];
    let plain = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0)).axes(false);
    let with = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .overlay(LineOverlay::new(&none));

    assert_eq!(
        render(&plain, 24, 12).content(),
        render(&with, 24, 12).content(),
        "an all-None line leaves the candles untouched"
    );
}

#[test]
fn line_overlay_autoscales_to_stay_in_view() {
    let candles = candles();
    let values = [Some(150.0), Some(150.0), Some(150.0)];

    let chart =
        CandlestickChart::new(CandleSeries::new(&candles)).overlay(LineOverlay::new(&values));
    let buf = render(&chart, 40, 16);
    assert!(count_fg(&buf, Color::White) > 0, "line should be visible");
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(text.contains("120"), "axis should expand toward the line");

    let pinned = CandlestickChart::new(CandleSeries::new(&candles))
        .overlay(LineOverlay::new(&values).autoscale(false));
    let ptext: String = render(&pinned, 40, 16)
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(
        !ptext.contains("120"),
        "axis should not expand when autoscale is off"
    );
}

#[test]
fn anchor_places_the_symbol_above_on_or_below_the_value() {
    let candles = candles();
    let items = [
        Annotation::new(1, 104.0).symbol("U").anchor(Anchor::Above),
        Annotation::new(1, 104.0).symbol("O").anchor(Anchor::On),
        Annotation::new(1, 104.0).symbol("D").anchor(Anchor::Below),
    ];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .overlay(Annotations::new(&items));
    let buf = render(&chart, 24, 14);

    let u = cell_of(&buf, "U").unwrap();
    let o = cell_of(&buf, "O").unwrap();
    let d = cell_of(&buf, "D").unwrap();
    assert!(
        u.1 < o.1 && o.1 < d.1,
        "Above < On < Below: {u:?} {o:?} {d:?}"
    );
    assert_eq!(u.0, o.0, "same column");
    assert_eq!(o.0, d.0, "same column");
}

#[test]
fn marker_aligns_with_its_candle_column_at_width_one() {
    let candles = candles();
    let items = [Annotation::new(1, 104.0).symbol("◆")];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(1.0).gap(1.0))
        .axes(false)
        .overlay(Annotations::new(&items));
    let buf = render(&chart, 20, 14);

    let diamond = cell_of(&buf, "◆").expect("marker drawn");
    // Width-1 candles tile every other column; candle index 1 sits in column 2,
    // not the gap at column 1 or 3.
    assert_eq!(diamond.0, 2, "marker should sit on the candle column");
}

#[test]
fn custom_symbol_and_style_are_honored() {
    let candles = candles();
    let items = [Annotation::new(1, 104.0).symbol("◆").style(Color::Cyan)];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .overlay(Annotations::new(&items));
    let buf = render(&chart, 24, 12);

    let diamond = cell_of(&buf, "◆").expect("custom symbol");
    assert_eq!(buf[diamond].fg, Color::Cyan);
}

#[test]
fn label_only_annotation_draws_text_with_no_symbol() {
    let candles = candles();
    let items = [Annotation::new(1, 104.0).label("HI")];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(3.0).gap(1.0))
        .axes(false)
        .overlay(Annotations::new(&items));
    let buf = render(&chart, 24, 12);

    assert!(cell_of(&buf, "H").is_some(), "label text present");
    assert!(cell_of(&buf, "▲").is_none(), "no marker glyph");
}

#[test]
fn off_screen_annotation_is_dropped_without_panic() {
    // Far more candles than fit, so the earliest scroll out of view.
    let candles: Vec<Candle> = (0..200)
        .map(|i| {
            let b = 100.0 + (i as f64 % 13.0);
            Candle::new(b, b + 2.0, b - 2.0, b + 1.0)
        })
        .collect();
    let items = [Annotation::new(0, 100.0).symbol("◆")];
    let chart = CandlestickChart::new(CandleSeries::new(&candles).width(1.0).gap(0.0))
        .axes(false)
        .overlay(Annotations::new(&items));
    let buf = render(&chart, 24, 12);

    assert!(
        cell_of(&buf, "◆").is_none(),
        "an annotation scrolled out of view is not drawn"
    );
}

#[test]
fn annotations_autoscale_to_keep_markers_in_view() {
    let candles = candles();
    let items = [Annotation::new(1, 150.0).symbol("◆")];

    let chart =
        CandlestickChart::new(CandleSeries::new(&candles)).overlay(Annotations::new(&items));
    let buf = render(&chart, 40, 16);
    assert!(cell_of(&buf, "◆").is_some(), "marker should be visible");
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(text.contains("120"), "axis should expand toward the marker");

    let pinned = CandlestickChart::new(CandleSeries::new(&candles))
        .overlay(Annotations::new(&items).autoscale(false));
    let ptext: String = render(&pinned, 40, 16)
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(
        !ptext.contains("120"),
        "axis should not expand when autoscale is off"
    );
}
