use crossterm::cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, SetCursorStyle};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use std::io;
use std::io::stdout;

const HELP: &str = r#"Testing the crossterm crate:
 - ESCAPE to quit!
 - Arrow keys to move.
 - s,v,u to set cursor style to SquareBlock, VerticalBar, UnderScore
 - b to toggle cursor blinking
 - g to goto the lower-right
 - ESCAPE to quit!
"#;

#[derive(Debug, Clone, Copy)]
enum CursorShape {
    SquareBlock,
    VerticalBar,
    UnderScore,
}

#[derive(Debug, Clone, Copy)]
struct CursorStyle {
    shape: CursorShape,
    blinking: bool,
}

impl CursorStyle {
    fn new() -> CursorStyle {
        CursorStyle {
            shape: CursorShape::SquareBlock,
            blinking: false,
        }
    }

    fn toggle_blinking(&mut self) -> Result<(), io::Error> {
        self.blinking = !self.blinking;
        self.enact()
    }

    fn set_shape(&mut self, shape: CursorShape) -> Result<(), io::Error> {
        self.shape = shape;
        self.enact()
    }

    fn enact(&self) -> Result<(), io::Error> {
        stdout().execute(self.crossterm_style())?;
        Ok(())
    }

    fn crossterm_style(&self) -> SetCursorStyle {
        use CursorShape::{SquareBlock, UnderScore, VerticalBar};
        use SetCursorStyle::{
            BlinkingBar, BlinkingBlock, BlinkingUnderScore, SteadyBar, SteadyBlock,
            SteadyUnderScore,
        };

        match (self.shape, self.blinking) {
            (SquareBlock, false) => SteadyBlock,
            (SquareBlock, true) => BlinkingBlock,
            (UnderScore, false) => SteadyUnderScore,
            (UnderScore, true) => BlinkingUnderScore,
            (VerticalBar, false) => SteadyBar,
            (VerticalBar, true) => BlinkingBar,
        }
    }
}

fn main() -> Result<(), io::Error> {
    println!("{}", HELP);

    fn is_key_event(event: &Event, key_code: KeyCode) -> bool {
        event == &Event::Key(KeyEvent::from(key_code))
    }

    enable_raw_mode()?;

    let mut cursor = CursorStyle::new();
    cursor.enact()?;

    loop {
        // Blocking read!
        let event = read()?;

        if is_key_event(&event, KeyCode::Char('s')) {
            cursor.set_shape(CursorShape::SquareBlock)?;
        } else if is_key_event(&event, KeyCode::Char('v')) {
            cursor.set_shape(CursorShape::VerticalBar)?;
        } else if is_key_event(&event, KeyCode::Char('u')) {
            cursor.set_shape(CursorShape::UnderScore)?;
        } else if is_key_event(&event, KeyCode::Char('b')) {
            cursor.toggle_blinking()?;
        } else if is_key_event(&event, KeyCode::Char('g')) {
            stdout().execute(MoveTo(500, 500))?;
        } else if is_key_event(&event, KeyCode::Up) {
            stdout().execute(MoveUp(1))?;
        } else if is_key_event(&event, KeyCode::Down) {
            stdout().execute(MoveDown(1))?;
        } else if is_key_event(&event, KeyCode::Right) {
            stdout().execute(MoveRight(1))?;
        } else if is_key_event(&event, KeyCode::Left) {
            stdout().execute(MoveLeft(1))?;
        }

        if event == Event::Key(KeyCode::Esc.into()) {
            break;
        }
    }

    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    disable_raw_mode()
}
