//! Render to and receive events from the terminal emulator.

mod screen_buf;
mod term_error;

pub use term_error::TermError;

use screen_buf::{ScreenBuf, ScreenOp};

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

use partial_pretty_printer::pane::PrettyWindow;
use partial_pretty_printer::{Col, Pos, Row, Size, Width};

use super::frontend::{Event, Frontend};
use super::key::Key;
use crate::style::{ColorTheme, ConcreteStyle, Rgb, Style};

/// Used to render to and receive events from the terminal emulator.
/// Implemented using [Termion](https://github.com/redox-os/termion).
/// Make only one.
pub struct Terminal {
    stdout: AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>,
    events: input::Events<Stdin>,
    color_theme: ColorTheme,
    buf: ScreenBuf,
    focus_pos: Option<Pos>,
}

impl Terminal {
    /// Get the current size of the actual terminal window, which might be different than the current size of the ScreenBuf.
    fn terminal_window_size() -> Result<Size, TermError> {
        let (width, height) = termion::terminal_size()?;
        Ok(Size {
            width,
            height: height as u32,
        })
    }

    /// Update the screen buffer size to match the actual terminal window size.
    /// If the screen buffer changes size as a result, its contents will be cleared.
    fn update_size(&mut self) -> Result<(), TermError> {
        let size = Self::terminal_window_size()?;
        if size != self.buf.size() {
            self.buf.resize(size);
        }
        Ok(())
    }

    fn write<T: Display>(&mut self, displayable: T) -> Result<(), io::Error> {
        write!(self.stdout, "{}", displayable)
    }

    fn go_to(&mut self, pos: Pos) -> Result<(), io::Error> {
        let (x, y) = pos_to_coords(pos);
        self.write(cursor::Goto(x, y))
    }

    fn apply_concrete_style(&mut self, style: ConcreteStyle) -> Result<(), io::Error> {
        if style.bold {
            self.write(Bold)?;
        } else {
            self.write(NoBold)?;
        }

        if style.underlined {
            self.write(Underline)?;
        } else {
            self.write(NoUnderline)?;
        }

        self.write(Fg(to_termion_rgb(style.fg_color)))?;
        self.write(Bg(to_termion_rgb(style.bg_color)))
    }
}

impl PrettyWindow for Terminal {
    type Error = TermError;
    type Style = Style;

    /// Return the current size of the screen buffer, without checking the
    /// actual size of the terminal window (which might have changed recently).
    fn size(&self) -> Result<Size, TermError> {
        Ok(self.buf.size())
    }

    /// Display a character at the given window position in the given style. `full_width` indicates
    /// whether the character is 1 (`false`) or 2 (`true`) columns wide. The character is guaranteed
    /// to fit in the window and not overlap or overwrite any other characters.
    fn display_char(
        &mut self,
        ch: char,
        pos: Pos,
        style: &Self::Style,
        full_width: bool,
    ) -> Result<(), Self::Error> {
        let width = if full_width { 2 } else { 1 };
        let concrete_style = self.color_theme.concrete_style(style);
        self.buf.display_char(ch, pos, concrete_style, width)
    }

    /// Invoked for each document for which [`PrintingOptions::set_focus`] is true,
    /// where `pos` is the focal point of the document.
    fn set_focus(&mut self, pos: Pos) -> Result<(), Self::Error> {
        self.focus_pos = Some(pos);
        Ok(())
    }
}

impl Frontend for Terminal {
    fn new(theme: ColorTheme) -> Result<Terminal, TermError> {
        let default_concrete_style = theme.concrete_style(&Style::default());
        let mut term = Terminal {
            stdout: AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode()?)),
            events: stdin().events(),
            color_theme: theme,
            buf: ScreenBuf::new(Terminal::terminal_window_size()?, default_concrete_style),
            focus_pos: None,
        };
        term.write(cursor::Hide)?;
        Ok(term)
    }

    fn next_event(&mut self) -> Option<Result<Event, TermError>> {
        match self.events.next() {
            Some(Ok(event::Event::Key(termion_key))) => Some(match Key::try_from(termion_key) {
                Ok(key) => Ok(Event::KeyEvent(key)),
                Err(()) => Err(TermError::UnknownKey),
            }),

            Some(Ok(event::Event::Mouse(event::MouseEvent::Press(
                event::MouseButton::Left,
                x,
                y,
            )))) => Some(Ok(Event::MouseEvent(coords_to_pos(x, y)))),
            Some(Ok(_)) => self.next_event(),
            Some(Err(err)) => Some(Err(err.into())),
            None => None,
        }
    }

    fn start_frame(&mut self) -> Result<(), TermError> {
        self.update_size()
    }

    fn show_frame(&mut self) -> Result<(), TermError> {
        // Reset terminal state
        self.write(Reset)?;
        self.write(cursor::Hide)?;
        // Update the screen from the old frame to the new frame.
        let changes: Vec<_> = self.buf.drain_changes().collect();
        for op in changes {
            match op {
                ScreenOp::Goto(pos) => self.go_to(pos)?,
                ScreenOp::Style(style) => self.apply_concrete_style(style)?,
                ScreenOp::Print(ch) => self.write(ch)?,
            }
        }
        if let Some(pos) = self.focus_pos.take() {
            self.go_to(pos)?;
            self.write(cursor::Show)?;
        }
        self.stdout.flush()?;
        Ok(())
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
    (pos.col + 1, pos.row as u16 + 1)
}

/// Convert termion's XY coordinates into a synless Pos.
fn coords_to_pos(x: u16, y: u16) -> Pos {
    Pos {
        col: x as Col - 1,
        row: y as Row - 1,
    }
}

impl TryFrom<termion::event::Key> for Key {
    type Error = ();
    fn try_from(termion_key: termion::event::Key) -> Result<Self, Self::Error> {
        Ok(match termion_key {
            termion::event::Key::Backspace => Key::Backspace,
            termion::event::Key::Left => Key::Left,
            termion::event::Key::Right => Key::Right,
            termion::event::Key::Up => Key::Up,
            termion::event::Key::Down => Key::Down,
            termion::event::Key::Home => Key::Home,
            termion::event::Key::End => Key::End,
            termion::event::Key::PageUp => Key::PageUp,
            termion::event::Key::PageDown => Key::PageDown,
            termion::event::Key::Delete => Key::Delete,
            termion::event::Key::Insert => Key::Insert,
            termion::event::Key::F(i) => Key::F(i),
            termion::event::Key::Char(c) => Key::Char(c),
            termion::event::Key::Alt(c) => Key::Alt(c),
            termion::event::Key::Ctrl(c) => Key::Ctrl(c),
            termion::event::Key::Null => Key::Null,
            termion::event::Key::Esc => Key::Esc,
            _ => return Err(()),
        })
    }
}