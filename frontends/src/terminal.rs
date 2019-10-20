//! Render to and receive events from the terminal emulator.

mod screen_buf;
mod term_error;
use screen_buf::{ScreenBuf, ScreenOp};
pub use term_error::TermError;

use std::convert::TryFrom;
use std::fmt::Display;
use std::io::{self, stdin, stdout, Stdin, Stdout, Write};

use termion::color::{Bg, Fg, Rgb as TermionRgb};
use termion::cursor;
use termion::event;
use termion::input::{self, MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::style::{Bold, NoBold, NoUnderline, Reset, Underline};

use pretty::{
    Col, ColorTheme, Pane, PaneError, Pos, PrettyWindow, Region, Rgb, Row, Shade, ShadedStyle,
    Style,
};

use crate::frontend::{Event, Frontend, Key};

use self::Event::{KeyEvent, MouseEvent};

/// Used to render to and receive events from the terminal emulator.
/// Implemented using [Termion](https://github.com/redox-os/termion).
/// Make only one.
pub struct Terminal {
    stdout: AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>,
    events: input::Events<Stdin>,
    color_theme: ColorTheme,
    buf: ScreenBuf,
}

impl Terminal {
    /// Update the screen buffer size to match the actual terminal window size.
    fn update_size(&mut self) -> Result<(), TermError> {
        let (col, row) = termion::terminal_size()?;
        let size = Pos {
            col: col as u16,
            row: row as u32,
        };

        if size != self.buf.size() {
            self.buf.resize(size);
        }
        Ok(())
    }

    fn write<T: Display>(&mut self, thing: T) -> Result<(), io::Error> {
        write!(self.stdout, "{}", thing)
    }

    fn go_to(&mut self, pos: Pos) -> Result<(), io::Error> {
        let (x, y) = pos_to_coords(pos);
        self.write(cursor::Goto(x, y))
    }

    fn apply_style(&mut self, style: ShadedStyle) -> Result<(), io::Error> {
        if style.emph.bold {
            self.write(Bold)?;
        } else {
            self.write(NoBold)?;
        }

        if style.emph.underlined {
            self.write(Underline)?;
        } else {
            self.write(NoUnderline)?;
        }

        self.write(Fg(to_termion_rgb(self.color_theme.foreground(style))))?;
        self.write(Bg(to_termion_rgb(self.color_theme.background(style))))
    }

    /// Prepare to start modifying a fresh new frame.
    fn start_frame(&mut self) -> Result<(), TermError> {
        self.update_size()
    }

    /// Show the modified frame to the user.
    fn show_frame(&mut self) -> Result<(), TermError> {
        // Reset terminal's style
        self.write(Reset)?;
        // Update the screen from the old frame to the new frame.
        let changes: Vec<_> = self.buf.drain_changes().collect();
        for op in changes {
            match op {
                ScreenOp::Goto(pos) => self.go_to(pos)?,
                ScreenOp::Apply(style) => self.apply_style(style)?,
                ScreenOp::Print(ch) => self.write(ch)?,
            }
        }
        self.stdout.flush()?;
        Ok(())
    }
}

impl PrettyWindow for Terminal {
    type Error = TermError;

    /// Return the current size of the screen buffer, without checking the
    /// actual size of the terminal window (which might have changed recently).
    fn size(&self) -> Result<Pos, Self::Error> {
        Ok(self.buf.size())
    }

    fn print(&mut self, pos: Pos, text: &str, style: Style) -> Result<(), Self::Error> {
        self.buf.write_str(pos, text, style)
    }

    fn highlight(
        &mut self,
        region: Region,
        shade: Option<Shade>,
        reverse: bool,
    ) -> Result<(), Self::Error> {
        self.buf.highlight(region, shade, reverse)
    }
}

impl Frontend for Terminal {
    type Error = TermError;
    type Window = Self;

    fn new(theme: ColorTheme) -> Result<Terminal, Self::Error> {
        let mut term = Terminal {
            stdout: AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode()?)),
            events: stdin().events(),
            color_theme: theme,
            buf: ScreenBuf::new(),
        };
        let size = term.size()?;
        term.buf.resize(size);
        term.write(cursor::Hide)?;
        Ok(term)
    }

    fn next_event(&mut self) -> Option<Result<Event, Self::Error>> {
        match self.events.next() {
            Some(Ok(event::Event::Key(termion_key))) => Some(match Key::try_from(termion_key) {
                Ok(key) => Ok(KeyEvent(key)),
                Err(()) => Err(TermError::UnknownKey),
            }),

            Some(Ok(event::Event::Mouse(event::MouseEvent::Press(
                event::MouseButton::Left,
                x,
                y,
            )))) => Some(Ok(MouseEvent(coords_to_pos(x, y)))),
            Some(Ok(_)) => self.next_event(),
            Some(Err(err)) => Some(Err(err.into())),
            None => None,
        }
    }

    fn draw_frame<F>(&mut self, draw: F) -> Result<(), PaneError>
    where
        F: Fn(Pane<Self>) -> Result<(), PaneError>,
    {
        self.start_frame().map_err(PaneError::from_pretty_window)?;
        let pane = self.pane().map_err(PaneError::from_pretty_window)?;
        let result = draw(pane);
        self.show_frame().map_err(PaneError::from_pretty_window)?;
        result
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.write(cursor::Show)
            .expect("failed to re-show cursor when dropping terminal")
    }
}

/// Convert the native synless Rgb type into the termion one. They're both
/// defined in different crates, so we can't impl From/Into.
fn to_termion_rgb(synless_rgb: Rgb) -> TermionRgb {
    TermionRgb(synless_rgb.red, synless_rgb.green, synless_rgb.blue)
}

/// Convert a synless Pos into termion's XY coordinates.
fn pos_to_coords(pos: Pos) -> (u16, u16) {
    (pos.col as u16 + 1, pos.row as u16 + 1)
}

/// Convert termion's XY coordinates into a synless Pos.
fn coords_to_pos(x: u16, y: u16) -> Pos {
    Pos {
        col: x as Col - 1,
        row: y as Row - 1,
    }
}
