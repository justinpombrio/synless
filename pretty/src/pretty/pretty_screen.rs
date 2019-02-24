use crate::geometry::{Bound, Pos, Region};
use crate::style::{Shade, Style};

/// A "screen" that supports the methods necessary to pretty-print a document.
///
/// To pretty-print, you need:
///
/// 1. A document that implements PrettyDocument, and
/// 2. A screen that implements PrettyScreen.
pub trait PrettyScreen {
    type Error;

    /// The size of the screen, as a `Bound`.
    fn size(&self) -> Result<Bound, Self::Error>;

    /// Display text.
    fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), Self::Error>;

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    fn shade(&mut self, region: Region, shade: Shade) -> Result<(), Self::Error>;

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`.
    fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), Self::Error>;

    /// If necessary, show the updated screen, e.g. by flipping a double buffer.
    fn show(&mut self) -> Result<(), Self::Error>;
}
