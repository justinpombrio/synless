use crate::geometry::{Col, Pos, Region, Row};

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub cursor_visibility: CursorVisibility,
    pub scroll_strategy: ScrollStrategy,
    pub width_strategy: WidthStrategy,
}

/// The visibility of the cursor in some document.
#[derive(Debug, Clone, Copy)]
pub enum CursorVisibility {
    Show,
    Hide,
}

/// How to choose the document width, after learning the how much width is available.
#[derive(Debug, Clone, Copy)]
pub enum WidthStrategy {
    /// Use all available width.
    Full,
    /// Use the given width.
    Fixed(Col),
    /// Try to use the given width. If that's not available, use as much width is available.
    NoMoreThan(Col),
}

impl WidthStrategy {
    pub fn choose(&self, available_width: Col) -> Col {
        match self {
            WidthStrategy::Full => available_width,
            WidthStrategy::Fixed(col) => *col,
            WidthStrategy::NoMoreThan(col) => (*col).min(available_width),
        }
    }
}

/// What part of the document to show, which may depend on the cursor position
#[derive(Debug, Clone, Copy)]
pub enum ScrollStrategy {
    /// Put this row and column of the document at the top left corner of the Pane.
    Fixed(Pos),
    /// Put the beginning of the document at the top left corner of the
    /// Pane. Equivalent to `Fixed(Pos{row: 0, col: 0})`.
    Beginning,
    /// Position the document such that the top of the cursor is at this height,
    /// where 1 is the top line of the Pane and 0 is the bottom line.
    CursorHeight { fraction: f32 },
}

impl ScrollStrategy {
    pub fn choose(self, available_height: Row, cursor_region: Region) -> Pos {
        match self {
            ScrollStrategy::CursorHeight { fraction } => {
                let fraction = f32::max(0.0, f32::min(1.0, fraction));
                let offset_from_top =
                    f32::round((available_height - 1) as f32 * (1.0 - fraction)) as Row;
                Pos {
                    col: 0,
                    row: u32::saturating_sub(cursor_region.pos.row, offset_from_top),
                }
            }
            ScrollStrategy::Fixed(pos) => pos,
            ScrollStrategy::Beginning => Pos { row: 0, col: 0 },
        }
    }
}
