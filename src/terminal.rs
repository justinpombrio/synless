//! Render to and poll events from the terminal emulator.

use coord::*;
use style::*;
use rustbox;
use rustbox::RustBox;
use rustbox::{InitOptions, InputMode, OutputMode, Mouse};
pub use self::Event::*;
pub use rustbox::Key;


/// Keyboard or mouse input
pub enum Event {
    KeyEvent(rustbox::keyboard::Key),
    /// (x, y) in window coordinates: (0, 0) is the upper-left.
    MouseEvent(i32, i32)
}

/// Used to render to and poll events from the terminal emulator.
/// Implemented using [Rustbox](https://github.com/gchp/rustbox).
/// Make only one.
pub struct Terminal {
    rust_box: RustBox
}

impl Terminal {
    pub fn new() -> Terminal {
        let settings = InitOptions{
            input_mode: InputMode::EscMouse,
            output_mode: OutputMode::EightBit,
            buffer_stderr: true
        };
        match RustBox::init(settings) {
            Result::Ok(v) => Terminal{ rust_box: v },
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
        let fg = style.foreground();
        let bg = style.background();
        let emph = style.emphasis();
        for (i, ch) in text.chars().enumerate() {
            let (row, col) = (pos.row as usize, pos.col as usize + i);
            self.rust_box.print_char(col, row, emph, fg, bg, ch);
        }
    }

    /// Render a character with the given style at the given position.
    /// No newlines allowed.
    pub fn print_char(&mut self, ch: char, pos: Pos, style: Style) {
        let fg = style.foreground();
        let bg = style.background();
        let emph = style.emphasis();
        let (row, col) = (pos.row as usize, pos.col as usize);
        self.rust_box.print_char(col, row, emph, fg, bg, ch);
    }

    /// Fill in a Region of the screen with the given
    /// background shade (and empty forground).
    pub fn shade_region(&mut self, region: Region, shade: Shade) {
        let fg = shade.into();
        let bg = shade.into();
        let emph = Emph::plain().into();

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
            col: 20,
//            col: self.rust_box.width() as Col,
            row: self.rust_box.height() as Row
        }
    }

    /// Poll for keyboard and mouse events. `None` means no event.
    pub fn poll_event(&self) -> Option<Event> {
        match self.rust_box.poll_event(false) {
            Ok(rustbox::Event::MouseEvent(Mouse::Left, x, y)) =>
                Some(MouseEvent(x, y)),
            Ok(rustbox::Event::KeyEvent(key)) =>
                Some(KeyEvent(key)),
            Ok(_) =>
                None,
            Err(e) =>
                panic!("Failed to poll terminal event!\n{}", e)
        }
    }
}
