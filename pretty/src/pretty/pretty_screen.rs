use crate::geometry::{Bound, Region, Pos};
use crate::style::{Shade, Style};


pub trait PrettyScreen {
    type Error;

    fn size(&self) -> Result<Bound, Self::Error>;
    fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), Self::Error>;
    fn shade(&mut self, region: Region, shade: Shade)       -> Result<(), Self::Error>;
    fn highlight(&mut self, pos: Pos, style: Style)         -> Result<(), Self::Error>;
}
