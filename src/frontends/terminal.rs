//! Render to and receive events from a terminal emulator.

use super::frontend::{Event, Frontend, Key, KeyCode, KeyModifiers, MouseButton, MouseEvent};
use super::screen_buf::{ScreenBuf, ScreenOp};
use crate::style::{ColorTheme, Rgb, Style};

use partial_pretty_printer::pane::PrettyWindow;
use partial_pretty_printer::{Col, Height, Pos, Row, Size};

use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};

use crossterm::cursor;
use crossterm::event as ct_event;
use crossterm::style::{
    Attribute, Attributes, Color, ResetColor, SetAttribute, SetAttributes, SetBackgroundColor,
    SetForegroundColor,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size as ct_size, BeginSynchronizedUpdate,
    EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::QueueableCommand;

/// Used to render to and receive events from the terminal emulator.
/// Implemented using [Crossterm](https://github.com/crossterm-rs/crossterm).
/// Make only one.
pub struct Terminal {
    color_theme: ColorTheme,
    buf: ScreenBuf,
    /// Where to place the terminal cursor. If `None`, hide the cursor.
    focus_pos: Option<Pos>,
}

#[derive(thiserror::Error, Debug)]
pub enum TerminalError {
    #[error("Terminal input/output error: {0}")]
    Io(#[from] io::Error),

    #[error("Character position outside window boundary")]
    OutOfBounds,
}

impl Terminal {
    pub fn new(theme: ColorTheme) -> Result<Terminal, TerminalError> {
        let default_concrete_style = theme.concrete_style(&Style::default());

        let mut term = Terminal {
            color_theme: theme,
            buf: ScreenBuf::new(Terminal::terminal_window_size()?, default_concrete_style),
            focus_pos: None,
        };
        term.enter()?;
        Ok(term)
    }

    /// Get the current size of the actual terminal window in characters. This may be different
    /// than the current size of the ScreenBuf.
    fn terminal_window_size() -> Result<Size, TerminalError> {
        let (width, height) = ct_size()?;
        Ok(Size {
            width,
            height: height as Height,
        })
    }

    /// Prepare the terminal for use. This should be run once on startup.
    fn enter(&mut self) -> Result<(), io::Error> {
        enable_raw_mode()?;
        stdout()
            .queue(EnterAlternateScreen)?
            .queue(cursor::SetCursorStyle::SteadyBar)?
            .queue(cursor::Hide)?;
        stdout().flush()
    }

    /// Reset the terminal. This should be run once on exit.
    fn exit(&mut self) -> Result<(), io::Error> {
        disable_raw_mode()?;
        stdout()
            .queue(LeaveAlternateScreen)?
            .queue(cursor::SetCursorStyle::DefaultUserShape)?
            .queue(cursor::Show)?
            .queue(ResetColor)?
            .queue(SetAttribute(Attribute::Reset))?;
        stdout().flush()
    }
}

impl PrettyWindow for Terminal {
    type Error = TerminalError;
    type Style = Style;

    // Return the current size of the screen buffer, without checking the
    // actual size of the terminal window (which might have changed recently).
    fn size(&self) -> Result<Size, TerminalError> {
        Ok(self.buf.size())
    }

    fn display_char(
        &mut self,
        ch: char,
        pos: Pos,
        style: &Self::Style,
        full_width: bool,
    ) -> Result<(), Self::Error> {
        let width = if full_width { 2 } else { 1 };
        let concrete_style = self.color_theme.concrete_style(style);
        if self.buf.display_char(ch, pos, concrete_style, width) {
            Ok(())
        } else {
            Err(TerminalError::OutOfBounds)
        }
    }

    fn set_focus(&mut self, pos: Pos) -> Result<(), Self::Error> {
        self.focus_pos = Some(pos);
        Ok(())
    }
}

impl Frontend for Terminal {
    fn set_color_theme(&mut self, theme: ColorTheme) -> Result<(), Self::Error> {
        let default_concrete_style = theme.concrete_style(&Style::default());
        self.color_theme = theme;
        self.buf.set_blank_style(default_concrete_style);
        Ok(())
    }

    fn next_event(&mut self, timeout: Duration) -> Result<Option<Event>, TerminalError> {
        let deadline = Instant::now() + timeout;
        let mut remaining = timeout;
        loop {
            if !ct_event::poll(remaining)? {
                return Ok(None);
            }
            let event = ct_event::read()?;
            if let Ok(relevant_event) = event.try_into() {
                return Ok(Some(relevant_event));
            }
            if let Some(t) = deadline.checked_duration_since(Instant::now()) {
                remaining = t;
            } else {
                return Ok(None);
            }
        }
    }

    fn start_frame(&mut self) -> Result<(), TerminalError> {
        // Update the screen buffer size to match the actual terminal window size.
        // If the screen buffer changes size as a result, its contents will be cleared.
        let size = Self::terminal_window_size()?;
        if size != self.buf.size() {
            self.buf.resize(size);
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), TerminalError> {
        fn move_to(pos: Pos) -> cursor::MoveTo {
            cursor::MoveTo(pos.col, pos.row as u16)
        }

        let mut out = stdout().lock();
        out.queue(BeginSynchronizedUpdate)?;

        for op in self.buf.drain_changes() {
            match op {
                // Assuming that ppp and the terminal agree about char width!
                ScreenOp::Print(ch, _) => write!(out, "{}", ch)?,
                ScreenOp::Goto(pos) => {
                    out.queue(move_to(pos))?;
                }
                ScreenOp::Style(style) => {
                    let mut attributes = Attributes::default();
                    if style.bold {
                        attributes.set(Attribute::Bold);
                    } else {
                        attributes.set(Attribute::NormalIntensity);
                    }
                    if style.underlined {
                        attributes.set(Attribute::Underlined);
                    } else {
                        attributes.set(Attribute::NoUnderline);
                    }
                    out.queue(SetAttributes(attributes))?;
                    out.queue(SetForegroundColor(style.fg_color.into()))?;
                    out.queue(SetBackgroundColor(style.bg_color.into()))?;
                }
            }
        }
        if let Some(pos) = self.focus_pos.take() {
            out.queue(move_to(pos))?;
            out.queue(cursor::Show)?;
        } else {
            out.queue(cursor::Hide)?;
        }

        out.queue(EndSynchronizedUpdate)?;
        out.flush()?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.exit()
            .expect("failed to restore terminal state on exit");
    }
}

/// Converts synless's `Rgb` to crossterm's `Color`
impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Color {
        Color::Rgb {
            r: rgb.red,
            g: rgb.green,
            b: rgb.blue,
        }
    }
}

impl TryInto<Event> for ct_event::Event {
    type Error = ();

    /// Returns `Err` if the event is irrelevant to us.
    fn try_into(self) -> Result<Event, ()> {
        match self {
            ct_event::Event::FocusGained | ct_event::Event::FocusLost => Err(()),
            ct_event::Event::Paste(s) => Ok(Event::Paste(s)),
            ct_event::Event::Resize(..) => Ok(Event::Resize),
            ct_event::Event::Mouse(mouse_event) => Ok(Event::Mouse(mouse_event.try_into()?)),
            ct_event::Event::Key(key_event) => Ok(Event::Key(key_event.try_into()?)),
        }
    }
}

impl TryInto<MouseEvent> for ct_event::MouseEvent {
    type Error = ();

    /// Returns `Err` if the event is irrelevant to us.
    fn try_into(self) -> Result<MouseEvent, ()> {
        if let ct_event::MouseEventKind::Down(ct_button) = self.kind {
            let button = match ct_button {
                ct_event::MouseButton::Left => MouseButton::Left,
                ct_event::MouseButton::Right => MouseButton::Right,
                ct_event::MouseButton::Middle => MouseButton::Middle,
            };
            Ok(MouseEvent {
                click_pos: Pos {
                    row: self.row as Row,
                    col: self.column as Col,
                },
                button,
            })
        } else {
            Err(())
        }
    }
}

impl TryInto<Key> for ct_event::KeyEvent {
    type Error = ();

    /// Returns `Err` if the event is irrelevant to us.
    fn try_into(self) -> Result<Key, ()> {
        if self.kind != ct_event::KeyEventKind::Press {
            return Err(());
        }
        let mut modifiers: KeyModifiers = self.modifiers.try_into()?;
        let code = match self.code {
            ct_event::KeyCode::Backspace => KeyCode::Backspace,
            ct_event::KeyCode::Enter => KeyCode::Enter,
            ct_event::KeyCode::Left => KeyCode::Left,
            ct_event::KeyCode::Right => KeyCode::Right,
            ct_event::KeyCode::Up => KeyCode::Up,
            ct_event::KeyCode::Down => KeyCode::Down,
            ct_event::KeyCode::Home => KeyCode::Home,
            ct_event::KeyCode::End => KeyCode::End,
            ct_event::KeyCode::PageUp => KeyCode::PageUp,
            ct_event::KeyCode::PageDown => KeyCode::PageDown,
            ct_event::KeyCode::Tab => KeyCode::Tab,
            ct_event::KeyCode::BackTab => {
                // Represent BackTab as shift+Tab, for normalization.
                modifiers.shift = true;
                KeyCode::Tab
            }
            ct_event::KeyCode::Delete => KeyCode::Delete,
            ct_event::KeyCode::Insert => KeyCode::Insert,
            ct_event::KeyCode::F(num) => KeyCode::F(num),
            ct_event::KeyCode::Char(c) => {
                if c.is_uppercase() {
                    // Remove redundant "shift", for normalization.
                    modifiers.shift = false;
                }
                KeyCode::Char(c)
            }
            ct_event::KeyCode::Esc => KeyCode::Esc,
            _ => {
                return Err(());
            }
        };
        Key::new(code, modifiers).ok_or(())
    }
}

impl TryInto<KeyModifiers> for ct_event::KeyModifiers {
    type Error = ();

    /// Returns Err if there's an unsupported modifier.
    fn try_into(self) -> Result<KeyModifiers, ()> {
        let mut mods = KeyModifiers {
            ctrl: false,
            alt: false,
            shift: false,
        };
        for flag in self.iter() {
            match flag {
                ct_event::KeyModifiers::CONTROL => {
                    mods.ctrl = true;
                }
                ct_event::KeyModifiers::ALT => {
                    mods.alt = true;
                }
                ct_event::KeyModifiers::SHIFT => {
                    mods.shift = true;
                }
                _ => {
                    return Err(());
                }
            }
        }
        Ok(mods)
    }
}
