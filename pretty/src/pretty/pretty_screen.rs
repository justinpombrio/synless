use crate::geometry::{Pos, Region};
use crate::style::{Shade, Style};

/// A "screen" that supports the methods necessary to pretty-print a document.
pub trait PrettyScreen {
    type Error;

    /// The size and location of the screen, as a `Region`.
    fn region(&self) -> Result<Region, Self::Error>;

    /// Display text.
    fn print(&mut self, offset: Pos, text: &str, style: Style) -> Result<(), Self::Error>;

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    fn shade(&mut self, region: Region, shade: Shade) -> Result<(), Self::Error>;

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`.
    fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), Self::Error>;
}
