use pretty::{ColorTheme, Pos, PrettyScreen, Region, Shade, Style};

pub use termion::event::Key;

// TODO: mouse events

/// An input event.
pub enum Event {
    /// A key was pressed down.
    KeyEvent(Key), // termion key. TODO: don't have trait depend on termion
    /// The left mouse button was pressed at the given character
    /// position (relative to the terminal window).
    MouseEvent(Pos),
}

/// A front end for the editor. It knows how to render to a screen,
/// and how to receive keyboard events.
pub trait Frontend: Sized {
    type Error;

    /// Construct a new frontend.
    fn new(theme: ColorTheme) -> Result<Self, Self::Error>;

    /// Iterate over all key and mouse events, blocking on `next()`.
    fn next_event(&mut self) -> Option<Result<Event, Self::Error>>;

    /// Return the current size of the screen in characters.
    fn size(&self) -> Result<Pos, Self::Error>;

    /// Prepare to start modifying a fresh new frame.
    fn start_frame(&mut self) -> Result<(), Self::Error>;

    /// Show the modified frame to the user.
    fn show_frame(&mut self) -> Result<(), Self::Error>;

    /// Get a FrontendScreen covering the frontend's full screen area.
    fn screen<'a>(&'a mut self) -> Result<FrontendScreen<'a, Self>, Self::Error> {
        let size = self.size()?;
        Ok(FrontendScreen {
            frontend: self,
            region: Region::new_rectangle(Pos::zero(), size),
        })
    }

    /// Render a string with the given style, with the first character at the
    /// given position. No newlines allowed. This is not intended to be called
    /// directly - it will be used for implementing the PrettyScreen trait.
    fn print_str(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), Self::Error>;

    /// Set the style within the given region. This is not intended to be called
    /// directly - it will be used for implementing the PrettyScreen trait.
    fn style_region(&mut self, region: Region, style: Style) -> Result<(), Self::Error>;

    /// Set the background shade within the given region. This is not intended to be called
    /// directly - it will be used for implementing the PrettyScreen trait.
    fn shade_region(&mut self, region: Region, shade: Shade) -> Result<(), Self::Error>;
}

/// Some region of the screen that PrettyDocuments can be rendered to.
pub struct FrontendScreen<'a, T>
where
    T: Frontend,
{
    frontend: &'a mut T,
    region: Region,
}

impl<'a, T> FrontendScreen<'a, T>
where
    T: Frontend,
{
    /// Convert relative position to absolute position
    fn abs(&self, rel: Pos) -> Pos {
        rel + self.region.pos
    }
}

impl<'a, T> PrettyScreen for FrontendScreen<'a, T>
where
    T: Frontend,
{
    type Error = T::Error;

    fn shade(&mut self, region: Region, shade: Shade) -> Result<(), Self::Error> {
        let abs_region = region + self.region.pos;
        self.frontend.shade_region(abs_region, shade)
    }

    fn highlight(&mut self, pos: Pos, style: Style) -> Result<(), Self::Error> {
        let abs_region = Region::char_region(self.abs(pos));
        self.frontend.style_region(abs_region, style)
    }

    fn region(&self) -> Result<Region, Self::Error> {
        Ok(self.region)
    }

    fn print(&mut self, offset: Pos, text: &str, style: Style) -> Result<(), Self::Error> {
        self.frontend.print_str(self.abs(offset), text, style)
    }
}
