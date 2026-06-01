//! Glyph family selection for candle rendering.

use crate::block::Block;
use crate::braille::Braille;
use crate::quadrant::Quadrant;
use crate::render::Rasterizer;

/// The glyph family a chart rasterizes candles with.
///
/// This mirrors Ratatui's [`symbols::Marker`](ratatui_core::symbols::Marker):
/// it selects which character set quantizes the fractional-row geometry into
/// terminal cells. Pass it to
/// [`CandlestickChart::marker`](crate::CandlestickChart::marker).
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Marker {
    /// Eighth-block glyphs (`█`, `▄`, ...). Bodies resolve to an eighth of a
    /// row and wick tips to a half, painting full-width bars.
    #[default]
    Block,
    /// [Braille](https://en.wikipedia.org/wiki/Braille_Patterns) dots, a 2x4
    /// grid per cell. This quadruples the vertical and doubles the horizontal
    /// resolution, at one color per cell.
    ///
    /// Support depends on the terminal and font; terminals without Unicode
    /// Braille show replacement glyphs instead of dots.
    Braille,
    /// Quadrant and half-block glyphs (`█`, `▀`, `▄`, `▌`, `▝`, ...), a 2x2 grid
    /// per cell. Filled bodies resolve to a half row; hollow bodies are drawn as
    /// a single-line box outline (`┌─┐`). Wick tips resolve to a half row, as
    /// with [`Block`](Self::Block).
    Quadrant,
}

impl Marker {
    /// The rasterizer backend that draws this glyph family.
    pub(crate) fn rasterizer(self) -> &'static dyn Rasterizer {
        match self {
            Self::Block => &Block,
            Self::Braille => &Braille,
            Self::Quadrant => &Quadrant,
        }
    }
}
