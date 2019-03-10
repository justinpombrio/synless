//! Render to and receive events from the terminal emulator.

use std::fmt::Display;
use std::io::{self, stdin, stdout, Stdin, Stdout, Write};

use termion::clear;
use termion::color::{Bg, Fg, Rgb as TermionRgb};
use termion::cursor;
use termion::event;
use termion::input::{self, MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::style::{Bold, NoBold, NoUnderline, Reset, Underline};

use pretty::{Bound, Col, Pos, Region, Row};
use pretty::{ColorTheme, PrettyScreen, Rgb, Shade, Style};

use crate::frontend::{Event, Frontend};

use self::Event::{KeyEvent, MouseEvent};

/// Used to render to and receive events from the terminal emulator.
/// Implemented using [Termion](https://github.com/redox-os/termion).
/// Make only one.
pub struct Terminal {
    stdout: AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>,
    events: input::Events<Stdin>,
    color_theme: ColorTheme,
}

impl Terminal {
    fn write<T: Display>(&mut self, thing: T) -> Result<(), io::Error> {
        write!(self.stdout, "{}", thing)
    }

    fn go_to(&mut self, pos: Pos) -> Result<(), io::Error> {
        let (x, y) = pos_to_coords(pos);
        self.write(cursor::Goto(x, y))
    }

    fn apply_style(&mut self, style: Style) -> Result<(), io::Error> {
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
}

impl PrettyScreen for Terminal {
    type Error = io::Error;

    fn region(&self) -> Result<Region, Self::Error> {
        let (cols, rows) = termion::terminal_size()?;
        Ok(Region {
            pos: Pos::zero(),
            bound: Bound::new_rectangle(rows as u32, cols),
        })
    }

    fn print(&mut self, offset: Pos, text: &str, style: Style) -> Result<(), Self::Error> {
        self.go_to(offset)?;
        self.apply_style(style)?;
        self.write(text)
    }

    fn shade(&mut self, _region: Region, _shade: Shade) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn highlight(&mut self, _pos: Pos, _style: Style) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn show(&mut self) -> Result<(), Self::Error> {
        self.stdout.flush()
    }
}

impl Frontend for Terminal {
    fn new(theme: ColorTheme) -> Result<Terminal, io::Error> {
        let mut term = Terminal {
            stdout: AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode()?)),
            events: stdin().events(),
            color_theme: theme,
        };
        term.write(cursor::Hide)?;
        Ok(term)
    }

    fn clear(&mut self) -> Result<(), io::Error> {
        // Reset style before clearing, or the most recently used background
        // color will fill the screen.
        self.write(Reset)?;
        self.write(clear::All)
    }

    fn next_event(&mut self) -> Option<Result<Event, io::Error>> {
        match self.events.next() {
            Some(Ok(event::Event::Key(key))) => Some(Ok(KeyEvent(key))),
            Some(Ok(event::Event::Mouse(event::MouseEvent::Press(
                event::MouseButton::Left,
                x,
                y,
            )))) => Some(Ok(MouseEvent(coords_to_pos(x, y)))),
            Some(Ok(_)) => self.next_event(),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }

    /*
    /// Fill in a Region of the screen with the given
    /// background shade (and empty forground).
    fn shade_region(&mut self, region: Region, shade: Shade) {
        let fg = self.color_theme.shade(shade);
        let bg = self.color_theme.shade(shade);
        let emph = self.color_theme.emph(Style::plain());

        let init_col   = region.pos.col as usize;
        let indent_col = (region.pos.col + region.indent()) as usize;
        let end_col    = (region.pos.col + region.width()) as usize;
        let init_row   = region.pos.row as usize;
        let end_row    = (region.pos.row + region.height()) as usize;

        for col in init_col .. end_col {
            for row in init_row .. end_row {
                self.rust_box.print_char(col, row, emph, fg, bg, ' ');
            }
        }
        for col in init_col .. indent_col {
            self.rust_box.print_char(col, end_row, emph, fg, bg, ' ');
        }
    }
     */
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
