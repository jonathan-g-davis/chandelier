//! Shared wick rendering.
//!
//! A wick is drawn as a vertical run of line glyphs along the candle's center column,
//! from the body out to the high and low.
//!
//! Used by the block and quadrant backends.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

use crate::render::CandleGeometry;

/// Vertical steps per terminal row a wick tip resolves to (`│`, `╵`, `╷`).
pub(crate) const HALVES_PER_ROW: u32 = 2;

/// Draws `geometry`'s wicks, reaching from the body's top and bottom cells
/// (`row_top`, `row_bot`) out to its high and low.
///
/// The high and low resolve to half a row, so a tip that lands mid-cell uses a
/// half-height glyph rather than snapping to a whole row. The wick is painted in
/// the candle's wick color over its background.
pub(crate) fn draw(
    buf: &mut Buffer,
    plot: Rect,
    geometry: &CandleGeometry,
    row_top: u32,
    row_bot: u32,
) {
    let CandleGeometry {
        high_row,
        low_row,
        wick: fg,
        bg,
        ..
    } = *geometry;

    // The wick runs along the whole column nearest the body's center.
    let center_col = plot.x + (geometry.center() - 0.5).round() as u16;

    let last_half = u32::from(plot.height) * HALVES_PER_ROW - 1;
    let high_half = (high_row * HALVES_PER_ROW as f64).round() as u32;
    let low_half = ((low_row * HALVES_PER_ROW as f64).round() as u32).min(last_half);

    draw_upper_wick(buf, plot, center_col, high_half, row_top, fg, bg);
    draw_lower_wick(buf, plot, center_col, low_half, row_bot, fg, bg);
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
