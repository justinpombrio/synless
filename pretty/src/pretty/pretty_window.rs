use std::fmt;

use crate::geometry::{Pos, Rect, Region};
use crate::pane::Pane;
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
