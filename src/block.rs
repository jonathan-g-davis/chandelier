//! Eighth-block rasterizer.
//!
//! A candle arrives as fractional row positions (from the price scale) plus the
//! colors to paint it in. This module owns everything specific to the block
//! character set: quantizing a row to one of eight vertical steps, the
//! foreground/background inversion that lets a body edge land between two rows,
//! and the line glyph used for wicks.
//!
//! Two properties are worth stating because they bound what the block set can
//! draw truthfully:
//!
//! - A body confined to a single row that touches neither the top nor the
//!   bottom of that cell is drawn flush to the cell bottom. Block glyphs fill
//!   from the bottom or from the top by inversion, but cannot float a segment
//!   in the middle of a cell. A body is never shorter than one eighth, so it
//!   always remains visible.
//! - Body endpoints resolve to an eighth of a row, wick endpoints to a half. A
//!   wick uses the vertical line glyphs `│`, `╵`, and `╷`, so its tip can land
//!   on a half-row boundary rather than snapping to a whole row.

use std::ops::Range;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

/// Vertical sub-cell steps per terminal row (one eighth-block each).
const EIGHTHS_PER_ROW: u32 = 8;

/// Vertical steps per terminal row a wick tip resolves to (`│`, `╵`, `╷`).
const HALVES_PER_ROW: u32 = 2;

/// Eighth-block fills indexed by how many eighths are lit from the bottom of the
/// cell (`0` is empty, `8` is full).
const EIGHTHS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// One candle's geometry and colors, in the rasterizer's own terms.
///
/// `cols` is the absolute column range the body spans and `center_col` the
/// absolute column carrying the wick. The four row fields are fractional rows
/// measured from the top of the plot (smaller is higher on screen):
/// `body_top_row`/`body_bottom_row` bound the body, `high_row`/`low_row` the
/// wicks. `body` paints the body, `wick` the wick, and `bg` is the color the
/// empty portion of a partially filled cell is drawn against.
pub(crate) struct CandleMarks {
    pub cols: Range<u16>,
    pub center_col: u16,
    pub body_top_row: f64,
    pub body_bottom_row: f64,
    pub high_row: f64,
    pub low_row: f64,
    pub body: Color,
    pub wick: Color,
    pub bg: Color,
}

/// Draws one candle into `plot`.
pub(crate) fn draw_candle(buf: &mut Buffer, plot: Rect, marks: &CandleMarks) {
    if plot.width == 0 || plot.height == 0 {
        return;
    }

    let CandleMarks {
        ref cols,
        center_col,
        body_top_row,
        body_bottom_row,
        high_row,
        low_row,
        body,
        wick,
        bg,
    } = *marks;

    let max_sub = u32::from(plot.height) * EIGHTHS_PER_ROW;

    // Body endpoints to the nearest eighth, at least one eighth tall so a doji
    // still shows a body.
    let mut top_sub = (body_top_row * EIGHTHS_PER_ROW as f64).round() as u32;
    let mut bot_sub = (body_bottom_row * EIGHTHS_PER_ROW as f64).round() as u32;

    if bot_sub <= top_sub {
        bot_sub = top_sub + 1;
    }
    bot_sub = bot_sub.min(max_sub);
    top_sub = top_sub.min(bot_sub - 1);

    let row_top = top_sub / EIGHTHS_PER_ROW;
    let row_bot = (bot_sub - 1) / EIGHTHS_PER_ROW;

    // Wicks reach from the body out to the high and low. A tip resolves to half
    // a row using the half-height vertical glyphs, so a high or low that lands
    // mid-cell is not snapped to a whole row.
    let last_half = u32::from(plot.height) * HALVES_PER_ROW - 1;
    let high_half = (high_row * HALVES_PER_ROW as f64).round() as u32;
    let low_half = ((low_row * HALVES_PER_ROW as f64).round() as u32).min(last_half);

    draw_upper_wick(buf, plot, center_col, high_half, row_top, wick, bg);
    draw_lower_wick(buf, plot, center_col, low_half, row_bot, wick, bg);

    let col_end = cols.end.min(plot.x + plot.width);

    for row in row_top..=row_bot {
        let cell_top = row * EIGHTHS_PER_ROW;
        let a = top_sub.max(cell_top) - cell_top;
        let b = bot_sub.min(cell_top + EIGHTHS_PER_ROW) - cell_top;

        if b <= a {
            continue;
        }

        let y = plot.y + row as u16;

        for cx in cols.start..col_end {
            set_body_cell(buf, cx, y, a as u16, b as u16, body, bg);
        }
    }
}

/// Fills the segment `[a, b)` (eighths from the top of the cell) at `(x, y)`
/// with `fill`, drawing the remaining (empty) eighths in the `empty` color.
///
/// A segment flush with the cell top is drawn with foreground/background
/// inversion so the lit eighths sit at the top; otherwise the eighths are lit
/// from the bottom.
fn set_body_cell(buf: &mut Buffer, x: u16, y: u16, a: u16, b: u16, fill: Color, empty: Color) {
    let eighths = EIGHTHS_PER_ROW as u16;

    let (symbol, fg, bg) = if b - a == eighths {
        // A full cell: a solid block, independent of the surrounding colors.
        ("█", fill, empty)
    } else if a == 0 {
        // Lit from the top: draw the unlit lower eighths in the empty color over
        // a fill-colored cell, so the top `b` eighths show the fill.
        (EIGHTHS[(eighths - b) as usize], empty, fill)
    } else {
        // Lit from the bottom (or a small floating segment): draw the lit
        // eighths directly, leaving the rest in the empty color.
        (EIGHTHS[(b - a) as usize], fill, empty)
    };

    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.fg = fg;
        cell.bg = bg;
    }
}

/// Draws the wick above the body, from the body's top cell up to `high_half`
/// (a half-row position from the plot top). The tip cell is the lower-half glyph
/// `╷` when the high lands on a half-row boundary, otherwise a full `│`.
fn draw_upper_wick(
    buf: &mut Buffer,
    plot: Rect,
    center_col: u16,
    high_half: u32,
    row_top: u32,
    fg: Color,
    bg: Color,
) {
    let body_edge = row_top * HALVES_PER_ROW;
    if high_half >= body_edge {
        return;
    }

    let tip = high_half / HALVES_PER_ROW;

    if !high_half.is_multiple_of(HALVES_PER_ROW) {
        // The high reaches only the lower half of the tip cell.
        set_wick(buf, plot, center_col, tip, "╷", fg, bg);
        for r in (tip + 1)..row_top {
            set_wick(buf, plot, center_col, r, "│", fg, bg);
        }
    } else {
        for r in tip..row_top {
            set_wick(buf, plot, center_col, r, "│", fg, bg);
        }
    }
}

/// Draws the wick below the body, from the body's bottom cell down to
/// `low_half`. The tip cell is the upper-half glyph `╵` when the low lands on a
/// half-row boundary, otherwise a full `│`.
fn draw_lower_wick(
    buf: &mut Buffer,
    plot: Rect,
    center_col: u16,
    low_half: u32,
    row_bot: u32,
    fg: Color,
    bg: Color,
) {
    let body_edge = (row_bot + 1) * HALVES_PER_ROW;
    if low_half < body_edge {
        return;
    }

    let tip = low_half / HALVES_PER_ROW;

    for r in (row_bot + 1)..tip {
        set_wick(buf, plot, center_col, r, "│", fg, bg);
    }

    if low_half.is_multiple_of(HALVES_PER_ROW) {
        // The low reaches only the upper half of the tip cell.
        set_wick(buf, plot, center_col, tip, "╵", fg, bg);
    } else {
        set_wick(buf, plot, center_col, tip, "│", fg, bg);
    }
}

/// Draws a wick glyph at `center_col`, row `r` of `plot`.
fn set_wick(
    buf: &mut Buffer,
    plot: Rect,
    center_col: u16,
    r: u32,
    symbol: &str,
    fg: Color,
    bg: Color,
) {
    if center_col >= plot.x + plot.width {
        return;
    }

    let y = plot.y + r as u16;

    if let Some(cell) = buf.cell_mut((center_col, y)) {
        cell.set_symbol(symbol);
        cell.fg = fg;
        cell.bg = bg;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: Color = Color::Rgb(0, 200, 120);
    const WICK: Color = Color::Rgb(110, 116, 130);
    const BG: Color = Color::Rgb(10, 10, 12);

    const PARTIAL: [&str; 7] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇"];

    fn buffer(w: u16, h: u16) -> Buffer {
        Buffer::empty(Rect::new(0, 0, w, h))
    }

    fn is_partial(symbol: &str) -> bool {
        PARTIAL.contains(&symbol)
    }

    /// A single-column candle at column 0, with the given rows and wick color.
    fn marks(top: f64, bottom: f64, high: f64, low: f64, wick: Color) -> CandleMarks {
        CandleMarks {
            cols: 0..1,
            center_col: 0,
            body_top_row: top,
            body_bottom_row: bottom,
            high_row: high,
            low_row: low,
            body: BODY,
            wick,
            bg: BG,
        }
    }

    #[test]
    fn full_cell_is_a_solid_block() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        // A body spanning the whole single row, no wicks.
        draw_candle(&mut buf, plot, &marks(0.0, 1.0, 0.0, 1.0, WICK));

        let cell = &buf[(0, 0)];
        assert_eq!(cell.symbol(), "█");
        assert_eq!(cell.fg, BODY);
    }

    #[test]
    fn flush_top_segment_inverts() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        // Top half of the cell only (rows 0.0 .. 0.5 => eighths 0..4).
        draw_candle(&mut buf, plot, &marks(0.0, 0.5, 0.0, 0.5, WICK));

        let cell = &buf[(0, 0)];
        assert!(is_partial(cell.symbol()), "got {:?}", cell.symbol());
        // Inversion: the body color sits in the background, lit eighths in empty.
        assert_eq!(cell.bg, BODY);
        assert_eq!(cell.fg, BG);
    }

    #[test]
    fn bottom_anchored_segment_lights_from_the_bottom() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        // Bottom half of the cell (rows 0.5 .. 1.0 => eighths 4..8).
        draw_candle(&mut buf, plot, &marks(0.5, 1.0, 0.5, 1.0, WICK));

        let cell = &buf[(0, 0)];
        assert!(is_partial(cell.symbol()), "got {:?}", cell.symbol());
        // Lit directly: the body is the foreground, the empty half the background.
        assert_eq!(cell.fg, BODY);
        assert_eq!(cell.bg, BG);
    }

    #[test]
    fn wick_floats_above_the_body_without_bleeding_into_its_edge() {
        let plot = Rect::new(0, 0, 1, 4);
        let mut buf = buffer(1, 4);

        // Body top has a partial edge in row 1 (top_sub 12 => a == 4); a high
        // above it puts a wick in row 0 that floats free of the body.
        draw_candle(&mut buf, plot, &marks(1.5, 3.0, 0.0, 4.0, WICK));

        // Row 0 carries the wick.
        assert_eq!(buf[(0, 0)].symbol(), "│");
        assert_eq!(buf[(0, 0)].fg, WICK);

        // The body's partial edge keeps the background in its empty half. The
        // wick color never leaks into the body cell, so the open/close stays at
        // the body/background boundary.
        let edge = &buf[(0, 1)];
        assert!(is_partial(edge.symbol()), "got {:?}", edge.symbol());
        assert_eq!(edge.fg, BODY);
        assert_eq!(edge.bg, BG);
    }

    #[test]
    fn lower_wick_tip_uses_a_half_glyph_when_the_low_lands_mid_cell() {
        let plot = Rect::new(0, 0, 1, 4);
        let mut buf = buffer(1, 4);

        // Body in row 0; the low sits at row 2.0 (a half-row boundary), so the
        // tip cell (row 2) shows only its upper half.
        draw_candle(&mut buf, plot, &marks(0.0, 1.0, 0.0, 2.0, WICK));

        assert_eq!(buf[(0, 1)].symbol(), "│", "full wick cell below the body");
        assert_eq!(buf[(0, 2)].symbol(), "╵", "half-row tip at the low");
        assert_eq!(buf[(0, 2)].fg, WICK);
    }

    #[test]
    fn upper_wick_tip_uses_a_half_glyph_when_the_high_lands_mid_cell() {
        let plot = Rect::new(0, 0, 1, 4);
        let mut buf = buffer(1, 4);

        // Body in row 3; the high sits at row 2.5 (a half-row boundary), so the
        // tip cell (row 2) shows only its lower half.
        draw_candle(&mut buf, plot, &marks(3.0, 4.0, 2.5, 4.0, WICK));

        assert_eq!(buf[(0, 2)].symbol(), "╷", "half-row tip at the high");
        assert_eq!(buf[(0, 2)].fg, WICK);
        assert_eq!(buf[(0, 3)].symbol(), "█", "solid body cell");
    }
}
