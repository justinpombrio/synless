//! Render to and poll events from the terminal emulator.

use geometry::*;
use style::*;
use rustbox;
use rustbox::RustBox;
use rustbox::{InitOptions, InputMode, OutputMode, Mouse};
pub use self::Event::*;
pub use rustbox::Key;


/// An input event.
pub enum Event {
    /// A key was pressed down.
    KeyEvent(rustbox::keyboard::Key),
    /// The left mouse button was pressed at the given character
    /// position (relative to the terminal window).
    MouseEvent(Pos)
}

/// Used to render to and poll events from the terminal emulator.
/// Implemented using [Rustbox](https://github.com/gchp/rustbox).
/// Make only one.
pub struct Terminal {
    rust_box: RustBox,
    color_theme: ColorTheme
}

impl Terminal {
    pub fn new(theme: ColorTheme) -> Terminal {
        let settings = InitOptions{
            input_mode: InputMode::EscMouse,
            output_mode: OutputMode::EightBit,
            buffer_stderr: true
        };
        match RustBox::init(settings) {
            Result::Ok(rb) => Terminal{
                rust_box: rb,
                color_theme: theme
            },
            Result::Err(e) => panic!("Failed to initialize Rustbox!\n{}", e),
        }
    }

    /// Flip the buffer to show the screen. Nothing will show up until
    /// you call this function!
    pub fn present(&mut self) {
        self.rust_box.present();
    }

    /// Render a string with plain style.
    /// No newlines allowed.
    pub fn simple_print(&mut self, text: &str, pos: Pos) {
        self.print_str(text, pos, Style::plain());
    }

    /// Render a string with the given style at the given position.
    /// No newlines allowed.
    pub fn print_str(&mut self, text: &str, pos: Pos, style: Style) {
        let fg = self.color_theme.foreground(style);
        let bg = self.color_theme.background(style);
        let emph = self.color_theme.emph(style);
        for (i, ch) in text.chars().enumerate() {
            let (row, col) = (pos.row as usize, pos.col as usize + i);
            self.rust_box.print_char(col, row, emph, fg, bg, ch);
        }
    }

    /// Render a character with the given style at the given position.
    /// No newlines allowed.
    pub fn print_char(&mut self, ch: char, pos: Pos, style: Style) {
        let fg = self.color_theme.foreground(style);
        let bg = self.color_theme.background(style);
        let emph = self.color_theme.emph(style);
        let (row, col) = (pos.row as usize, pos.col as usize);
        self.rust_box.print_char(col, row, emph, fg, bg, ch);
    }

    /// Fill in a Region of the screen with the given
    /// background shade (and empty forground).
    pub fn shade_region(&mut self, region: Region, shade: Shade) {
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

    /// Clear the whole screen to black.
    pub fn clear(&mut self) {
        self.rust_box.clear();
    }

    /// Return the current size of the terminal in characters.
    pub fn size(&self) -> Pos {
        Pos{
            col: self.rust_box.width() as Col,
            row: self.rust_box.height() as Row
        }
    }

    /// Poll for keyboard and mouse events. `None` means no event.
    pub fn poll_event(&self) -> Option<Event> {
        // Ctrl-n = Enter
        // Ctrl-r = Return?
        // Ctrl-m = Tab
        match self.rust_box.poll_event(false) {
            Ok(rustbox::Event::MouseEvent(Mouse::Left, x, y)) =>
                Some(MouseEvent(Pos{ col: x as Col, row: y as Row })),
            Ok(rustbox::Event::KeyEvent(key)) =>
                Some(KeyEvent(key)),
            Ok(_) =>
                None,
            Err(e) =>
                panic!("Failed to poll terminal event!\n{}", e)
        }
    }
}

// static mut N : usize = 0;
