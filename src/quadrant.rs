//! Quadrant-block rasterizer.
//!
//! Quadrant glyphs split a cell into a 2x2 grid, doubling both the vertical and
//! horizontal resolution of a whole cell.
//!
//! A hollow body is drawn as a box-drawing outline (`┌─┐`, `│ │`, `└─┘`) at
//! whole-cell resolution, giving a crisp single-line border instead of a thick
//! block edge. A body too small to enclose an outline (under two cells in either
//! dimension) falls back to a solid body.

use std::ops::Range;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

use crate::render::{BodyFill, CandleGeometry, Rasterizer};
use crate::wick;

/// Quadrant-block rasterizer backend.
///
/// Quantizes a candle's fractional rows to a 2x2 sub-cell grid and paints filled
/// bodies with quadrant and half-block glyphs, hollow bodies with a box-drawing
/// outline.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Quadrant;

impl Rasterizer for Quadrant {
    fn draw_candle(&self, buf: &mut Buffer, plot: Rect, geometry: &CandleGeometry) {
        draw_candle(buf, plot, geometry);
    }
}

/// Sub-cell rows per terminal row (the height of the 2x2 grid).
const SUB_Y: u32 = 2;

/// Lit-sub-cell bit positions within a cell's 2x2 grid.
const TOP_LEFT: u8 = 1 << 0;
const TOP_RIGHT: u8 = 1 << 1;
const BOTTOM_LEFT: u8 = 1 << 2;
const BOTTOM_RIGHT: u8 = 1 << 3;

/// Glyphs indexed by which of the four sub-cells are lit (the [`TOP_LEFT`] ..
/// [`BOTTOM_RIGHT`] bits).
const QUADRANTS: [&str; 16] = [
    " ", // ----
    "▘", // TL
    "▝", // TR
    "▀", // TL TR
    "▖", // BL
    "▌", // TL BL
    "▞", // TR BL
    "▛", // TL TR BL
    "▗", // BR
    "▚", // TL BR
    "▐", // TR BR
    "▜", // TL TR BR
    "▄", // BL BR
    "▙", // TL BL BR
    "▟", // TR BL BR
    "█", // TL TR BL BR
];

/// Draws one candle into `plot`.
pub(crate) fn draw_candle(buf: &mut Buffer, plot: Rect, geometry: &CandleGeometry) {
    if plot.width == 0 || plot.height == 0 {
        return;
    }

    let CandleGeometry {
        body_left,
        body_right,
        body_top_row,
        body_bottom_row,
        body,
        bg,
        fill,
        ..
    } = *geometry;

    let max_sub = u32::from(plot.height) * SUB_Y;

    // Body endpoints to the nearest half-row, at least one half tall so a doji
    // still shows a body.
    let mut top_sub = (body_top_row * SUB_Y as f64).round() as u32;
    let mut bot_sub = (body_bottom_row * SUB_Y as f64).round() as u32;
    if bot_sub <= top_sub {
        bot_sub = top_sub + 1;
    }
    bot_sub = bot_sub.min(max_sub);
    top_sub = top_sub.min(bot_sub - 1);

    let row_top = top_sub / SUB_Y;
    let row_bot = (bot_sub - 1) / SUB_Y;

    // Body edges to the nearest whole column, at least one column wide.
    let left_col = plot.x + body_left.round() as u16;
    let mut right_col = plot.x + body_right.round() as u16;
    if right_col <= left_col {
        right_col = left_col + 1;
    }
    let body_cols = left_col..right_col.min(plot.x + plot.width);

    wick::draw(buf, plot, geometry, row_top, row_bot);

    // A hollow body is a single-line box outline, but only when it is at least
    // two cells in each dimension so the border encloses something. Narrower or
    // shorter bodies have no room for an outline and stay solid.
    let outlineable = body_cols.len() >= 2 && row_bot > row_top;
    if fill == BodyFill::Hollow && outlineable {
        draw_outline(buf, plot, body_cols, row_top, row_bot, body, bg);
    } else {
        fill_body(buf, plot, body_cols, top_sub, bot_sub, body, bg);
    }
}

/// Fills the body `cols` over the half-rows `[top_sub, bot_sub)` with `body`,
/// choosing the quadrant glyph that matches each cell's lit sub-cells. A body
/// spans whole columns, so both the left and right sub-cells of every covered
/// half-row are lit.
fn fill_body(
    buf: &mut Buffer,
    plot: Rect,
    cols: Range<u16>,
    top_sub: u32,
    bot_sub: u32,
    body: Color,
    bg: Color,
) {
    let row_top = top_sub / SUB_Y;
    let row_bot = (bot_sub - 1) / SUB_Y;

    for row in row_top..=row_bot {
        let cell_top = row * SUB_Y;
        let mut bits = 0u8;
        if top_sub <= cell_top && cell_top < bot_sub {
            bits |= TOP_LEFT | TOP_RIGHT;
        }
        if top_sub <= cell_top + 1 && cell_top + 1 < bot_sub {
            bits |= BOTTOM_LEFT | BOTTOM_RIGHT;
        }

        let y = plot.y + row as u16;
        for x in cols.clone() {
            set_cell(buf, x, y, bits, body, bg);
        }
    }
}

/// Draws a box-drawing outline around the body `cols` x `row_top..=row_bot` in
/// `fg`, clearing the interior to `bg`.
fn draw_outline(
    buf: &mut Buffer,
    plot: Rect,
    cols: Range<u16>,
    row_top: u32,
    row_bot: u32,
    fg: Color,
    bg: Color,
) {
    let x_first = cols.start;
    let x_last = cols.end - 1;
    for row in row_top..=row_bot {
        let top = row == row_top;
        let bottom = row == row_bot;
        let y = plot.y + row as u16;

        for x in cols.clone() {
            let left = x == x_first;
            let right = x == x_last;
            let symbol = match (top, bottom, left, right) {
                (true, _, true, _) => "┌",
                (true, _, _, true) => "┐",
                (_, true, true, _) => "└",
                (_, true, _, true) => "┘",
                (true, ..) | (_, true, ..) => "─",
                (_, _, true, _) | (_, _, _, true) => "│",
                _ => " ",
            };
            let border = top || bottom || left || right;

            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(symbol);
                cell.fg = if border { fg } else { bg };
                cell.bg = bg;
            }
        }
    }
}

/// Sets the quadrant glyph for the lit sub-cells `bits` at `(x, y)`, in `fg`
/// over `bg`. An empty cell is left untouched.
fn set_cell(buf: &mut Buffer, x: u16, y: u16, bits: u8, fg: Color, bg: Color) {
    if bits == 0 {
        return;
    }
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(QUADRANTS[bits as usize]);
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

    fn buffer(w: u16, h: u16) -> Buffer {
        Buffer::empty(Rect::new(0, 0, w, h))
    }

    /// A single-column filled candle at column 0.
    fn geometry(top: f64, bottom: f64, high: f64, low: f64) -> CandleGeometry {
        CandleGeometry {
            body_left: 0.0,
            body_right: 1.0,
            body_top_row: top,
            body_bottom_row: bottom,
            high_row: high,
            low_row: low,
            body: BODY,
            wick: WICK,
            bg: BG,
            fill: BodyFill::Filled,
        }
    }

    #[test]
    fn full_cell_is_a_solid_block() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        draw_candle(&mut buf, plot, &geometry(0.0, 1.0, 0.0, 1.0));

        let cell = &buf[(0, 0)];
        assert_eq!(cell.symbol(), "█");
        assert_eq!(cell.fg, BODY);
    }

    #[test]
    fn top_half_is_an_upper_half_block() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        // Body covers the top half-row (rows 0.0 .. 0.5 => half 0).
        draw_candle(&mut buf, plot, &geometry(0.0, 0.5, 0.0, 0.5));

        let cell = &buf[(0, 0)];
        assert_eq!(cell.symbol(), "▀");
        assert_eq!(cell.fg, BODY);
        assert_eq!(cell.bg, BG);
    }

    #[test]
    fn bottom_half_is_a_lower_half_block() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        // Body covers the bottom half-row (rows 0.5 .. 1.0 => half 1).
        draw_candle(&mut buf, plot, &geometry(0.5, 1.0, 0.5, 1.0));

        let cell = &buf[(0, 0)];
        assert_eq!(cell.symbol(), "▄");
        assert_eq!(cell.fg, BODY);
    }

    #[test]
    fn partial_body_renders_over_a_transparent_background() {
        let plot = Rect::new(0, 0, 1, 1);
        let mut buf = buffer(1, 1);

        // A top-half body over a Reset (terminal default) background: the half
        // block already lights its top half in the foreground, so the body color
        // stays in the foreground and the empty half resolves to the terminal
        // default. No inversion is needed.
        let geometry = CandleGeometry {
            bg: Color::Reset,
            ..geometry(0.0, 0.5, 0.0, 0.5)
        };
        draw_candle(&mut buf, plot, &geometry);

        let cell = &buf[(0, 0)];
        assert_eq!(cell.symbol(), "▀");
        assert_eq!(cell.fg, BODY);
        assert_eq!(cell.bg, Color::Reset);
    }

    #[test]
    fn wick_uses_the_block_line_glyphs() {
        let plot = Rect::new(0, 0, 1, 4);
        let mut buf = buffer(1, 4);

        // Body in row 0; the low sits at row 2.0 (a half-row boundary), so the
        // tip cell shows the upper-half glyph, exactly as the block backend.
        draw_candle(&mut buf, plot, &geometry(0.0, 1.0, 0.0, 2.0));

        assert_eq!(buf[(0, 1)].symbol(), "│", "full wick cell below the body");
        assert_eq!(buf[(0, 2)].symbol(), "╵", "half-row tip at the low");
        assert_eq!(buf[(0, 2)].fg, WICK);
    }

    /// A candle at columns `[x0, x1)` spanning the given rows, hollow.
    fn hollow(x0: u16, x1: u16, top: f64, bottom: f64) -> CandleGeometry {
        CandleGeometry {
            body_left: f64::from(x0),
            body_right: f64::from(x1),
            body_top_row: top,
            body_bottom_row: bottom,
            high_row: top,
            low_row: bottom,
            body: BODY,
            wick: WICK,
            bg: BG,
            fill: BodyFill::Hollow,
        }
    }

    #[test]
    fn hollow_body_draws_a_box_drawing_outline() {
        let plot = Rect::new(0, 0, 3, 3);
        let mut buf = buffer(3, 3);

        // A three-wide, full-height body.
        draw_candle(&mut buf, plot, &hollow(0, 3, 0.0, 3.0));

        let grid: Vec<String> = (0..3)
            .map(|y| (0..3).map(|x| buf[(x, y)].symbol()).collect())
            .collect();
        assert_eq!(grid, ["┌─┐", "│ │", "└─┘"]);

        // The border is drawn in the body color; the interior is cleared.
        assert_eq!(buf[(0, 0)].fg, BODY);
        assert_eq!(buf[(1, 1)].symbol(), " ");
        assert_eq!(buf[(1, 1)].bg, BG);
    }

    #[test]
    fn hollow_two_by_two_is_just_corners() {
        let plot = Rect::new(0, 0, 2, 2);
        let mut buf = buffer(2, 2);

        draw_candle(&mut buf, plot, &hollow(0, 2, 0.0, 2.0));

        let grid: Vec<String> = (0..2)
            .map(|y| (0..2).map(|x| buf[(x, y)].symbol()).collect())
            .collect();
        assert_eq!(grid, ["┌┐", "└┘"]);
    }

    #[test]
    fn hollow_falls_back_to_a_solid_body_below_two_columns() {
        let plot = Rect::new(0, 0, 1, 3);
        let mut buf = buffer(1, 3);

        // One column has no room for left and right walls, so the body stays
        // solid full blocks.
        draw_candle(&mut buf, plot, &hollow(0, 1, 0.0, 3.0));

        for y in 0..3 {
            assert_eq!(buf[(0, y)].symbol(), "█", "cell (0, {y}) stays solid");
        }
    }

    #[test]
    fn hollow_falls_back_to_a_solid_body_when_one_cell_tall() {
        let plot = Rect::new(0, 0, 3, 1);
        let mut buf = buffer(3, 1);

        // A single cell row cannot hold a top and a bottom edge, so the body is
        // drawn solid.
        draw_candle(&mut buf, plot, &hollow(0, 3, 0.0, 1.0));

        for x in 0..3 {
            assert_eq!(buf[(x, 0)].symbol(), "█", "cell ({x}, 0) stays solid");
        }
    }
}
