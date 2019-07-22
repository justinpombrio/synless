use std::fmt;

use crate::geometry::{Pos, Rect, Region};
use crate::style::{Shade, Style};

/// A "window" that supports the methods necessary to render a set of [PrettyDocument]s.
pub trait PrettyWindow: Sized {
    type Error: fmt::Debug;

    /// The size of the window, in rows and columns of characters.
    fn size(&self) -> Result<Pos, Self::Error>;

    /// Render a string with the given style, with the first character at the
    /// given position. No newlines allowed.
    fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), Self::Error>;

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    fn shade(&mut self, region: Region, shade: Shade) -> Result<(), Self::Error>;

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`.
    fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), Self::Error>;

    /// Get a `Pane` that covers the full window area (and can be pretty-printed to).
    fn pane<'a>(&'a mut self) -> Result<Pane<'a, Self>, Self::Error> {
        let rect = Rect::new(Pos::zero(), self.size()?);
        Ok(Pane { window: self, rect })
    }
}

/// A rectangular area of a window. You can pretty-print to it, or get sub-panes
/// of it and pretty-print to those.
pub struct Pane<'a, T>
where
    T: PrettyWindow,
{
    window: &'a mut T,
    rect: Rect,
}

impl<'a, T> Pane<'a, T>
where
    T: PrettyWindow,
{
    /// Get the position and size of the rectangular area covered by this `Pane`.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get a new `Pane` representing only the given sub-region of this `Pane`.
    /// Returns `None` if `rect` is not fully contained within this `Pane`.
    /// `rect` is specified in the same absolute coordinate system as the full
    /// `PrettyWindow` (not specified relative to this `Pane`!).
    pub fn sub_pane<'b>(&'b mut self, rect: Rect) -> Option<Pane<'b, T>> {
        if !self.rect().covers(rect) {
            return None;
        }
        Some(Pane {
            window: self.window,
            rect,
        })
    }

    /// Render a string with the given style, with its first character at the
    /// given relative position (where 0,0 is the top left corner of the
    /// `Pane`). No newlines allowed.
    pub fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.rect.pos();
        self.window.print(abs_pos, text, style)
    }

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    /// The region position is relative to the `Pane` (where 0,0 is the
    /// top left corner of the `Pane`).
    pub fn shade(&mut self, region: Region, shade: Shade) -> Result<(), T::Error> {
        let abs_region = region + self.rect.pos();
        self.window.shade(abs_region, shade)
    }

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`. The
    /// position is relative to the `Pane` (where 0,0 is the top left
    /// corner of the `Pane`).
    pub fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.rect.pos();
        self.window.highlight(abs_pos, style)
    }
}
