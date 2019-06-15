use crate::geometry::{Bound, Pos, Region};
use crate::style::{Shade, Style};

/// A "screen" that supports the methods necessary to pretty-print a document.
pub trait PrettyScreen: Sized {
    type Error;

    /// The size and shape of the screen, as a `Bound`.
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

    /// Get a ScreenRegion covering the full screen area.
    fn screen<'a>(&'a mut self) -> Result<ScreenRegion<'a, Self>, Self::Error> {
        let full_region = Region {
            pos: Pos::zero(),
            bound: self.bound()?,
        };

        Ok(ScreenRegion {
            master: self,
            region: full_region,
        })
    }
}

pub struct ScreenRegion<'a, T>
where
    T: PrettyScreen,
{
    master: &'a mut T,
    region: Region,
}

impl<'a, T> ScreenRegion<'a, T>
where
    T: PrettyScreen,
{
    /// Get the size and shape of this screen region.
    pub fn bound(&self) -> Bound {
        self.region.bound
    }

    /// Render a string with the given style, with the first character at the
    /// given position. No newlines allowed.
    pub fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.region.pos;
        self.master.print(abs_pos, text, style)
    }

    /// Shade the background. It is possible that the same position will be
    /// shaded more than once, or will be `.print`ed before being shaded. If so,
    /// the new shade should override the background color, but not the text.
    pub fn shade(&mut self, region: Region, shade: Shade) -> Result<(), T::Error> {
        let abs_region = region + self.region.pos;
        self.master.shade(abs_region, shade)
    }

    /// Shade a particular character position. This is used to highlight the
    /// cursor position while in text mode. It should behave the same way as
    /// `.shade` would with a small Region that included just `pos`.
    pub fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), T::Error> {
        let abs_pos = pos + self.region.pos;
        self.master.highlight(abs_pos, style)
    }

    /// Get a new `ScreenRegion` representing only the given sub-region of this
    /// `ScreenRegion`. `region` is specified in a coordinate system in which this
    /// screen's upper left corner is the origin. Returns `None` if `region` is
    /// not contained within this screen's bound.
    pub fn sub_screen<'b>(&'b mut self, region: Region) -> Option<ScreenRegion<'b, T>> {
        let full_region = Region {
            pos: Pos::zero(),
            bound: self.bound(),
        };
        if !full_region.covers(region) {
            return None;
        }

        Some(ScreenRegion {
            master: self.master,
            region,
        })
    }
}
