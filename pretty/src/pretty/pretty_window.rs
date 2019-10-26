use crate::geometry::{Pos, Rect, Region};
use crate::pane::Pane;
use crate::style::{Shade, Style};

/// A "window" that supports the methods necessary to render a set of [PrettyDocument](crate::PrettyDocument)s.
pub trait PrettyWindow: Sized {
    // Forbid the Error type from containing non-static references so we can use
    // it as a trait object.
    type Error: std::error::Error + 'static;

    /// The size of the window, in rows and columns of characters.
    fn size(&self) -> Result<Pos, Self::Error>;

    /// Render a string with the given style, with the first character at the
    /// given position. No newlines allowed.
    fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), Self::Error>;

    /// Highlight the region by shading and/or reversing it. If `shade` is `Some`,
    /// set the region's background color to that `Shade`. If `reverse`
    /// is true, toggle whether the foreground and background colors are swapped
    /// within the region.
    fn highlight(
        &mut self,
        region: Region,
        shade: Option<Shade>,
        reverse: bool,
    ) -> Result<(), Self::Error>;

    /// Get a `Pane` that covers the full window area (and can be pretty-printed to).
    fn pane<'a>(&'a mut self) -> Result<Pane<'a, Self>, Self::Error> {
        let rect = Rect::new(Pos::zero(), self.size()?);
        Ok(Pane { window: self, rect })
    }
}
