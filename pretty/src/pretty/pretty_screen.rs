use std::fmt;

use crate::geometry::{Bound, Pos, Region};
use crate::style::{Shade, Style};

/// A "window" that supports the methods necessary to pretty-print a document.
pub trait PrettyWindow: Sized {
    type Error: fmt::Debug;

    /// The size and shape of the window, as a `Bound`.
    fn bound(&self) -> Result<Bound, Self::Error>;

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

    /// Get a Pane covering the full window area.
    fn pane<'a>(&'a mut self) -> Result<Pane<'a, Self>, Self::Error> {
        let full_region = Region {
            pos: Pos::zero(),
            bound: self.bound()?,
        };

        Ok(Pane {
            window: self,
            region: full_region,
        })
    }
}

/// A region of a window. It can be further split into sub-panes.
pub struct Pane<'a, T>
where
    T: PrettyWindow,
{
    window: &'a mut T,
    region: Region,
}

/// A `PrettyPane` is like a `Pane`, except you can pretty-print to it, and you can't split it into sub-panes.
pub struct PrettyPane<'a, T>
where
    T: PrettyWindow,
{
    window: &'a mut T,
    region: Region,
}

impl<'a, T> Pane<'a, T>
where
    T: PrettyWindow,
{
    /// Get the position, size and shape of the region covered by this `Pane`.
    pub fn region(&self) -> Region {
        self.region
    }

    /// Get a `PrettyPane` that can be used to pretty-print to the region
    /// covered by this `Pane`.
    pub fn pretty_pane<'b>(&'b mut self) -> PrettyPane<'b, T> {
        PrettyPane {
            window: self.window,
            region: self.region,
        }
    }

    /// Get a new `Pane` representing only the given sub-region of this `Pane`.
    /// Returns `None` if `region` is not fully contained within this `Pane`.
    /// `region` is specified in the same absolute coordinate system as the full
    /// `PrettyWindow` (not specified relative to this `Pane`!).
    pub fn sub_pane<'b>(&'b mut self, region: Region) -> Option<Pane<'b, T>> {
        if !self.region().covers(region) {
            return None;
        }
        Some(Pane {
            window: self.window,
            region,
        })
    }
}

impl<'a, T> PrettyPane<'a, T>
where
    T: PrettyWindow,
{
    /// Get the size and shape of the region covered by this `PrettyPane`.
    pub fn bound(&self) -> Bound {
        self.region.bound
    }

    /// Render a string with the given style, with its first character at the
    /// given relative position (where 0,0 is the top left corner of the
    /// `PrettyPane`). No newlines allowed.
    pub fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.region.pos;
        self.window.print(abs_pos, text, style)
    }

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    /// The region position is relative to the `PrettyPane` (where 0,0 is the
    /// top left corner of the `PrettyPane`).
    pub fn shade(&mut self, region: Region, shade: Shade) -> Result<(), T::Error> {
        let abs_region = region + self.region.pos;
        self.window.shade(abs_region, shade)
    }

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`. The
    /// position is relative to the `PrettyPane` (where 0,0 is the top left
    /// corner of the `PrettyPane`).
    pub fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.region.pos;
        self.window.highlight(abs_pos, style)
    }
}
