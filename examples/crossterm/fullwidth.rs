use crossterm::cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp};
use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, DisableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;

const HELP: &str = r#"Testing the crossterm crate:
 - Arrow keys to move.
 - letters to type half-width ascii letters
 - digits to type full-width Chinese numerals
 - ESCAPE to quit!
"#;

const DIGITS: &[char; 10] = &['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

fn start() -> Result<(), io::Error> {
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, DisableLineWrap, MoveTo(0, 0))?;
    for line in HELP.lines() {
        println!("{}", line);
        execute!(stdout, MoveLeft(100))?;
    }
    execute!(stdout, MoveTo(0, 500))
}

fn finish() -> Result<(), io::Error> {
    let mut stdout = io::stdout();

    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()
}

fn main() -> Result<(), io::Error> {
    use std::io::Write;

    start()?;

    let mut stdout = io::stdout();
    loop {
        // Blocking read!
        let event = read()?;

        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Esc => break,
                KeyCode::Up => execute!(stdout, MoveUp(1))?,
                KeyCode::Down => execute!(stdout, MoveDown(1))?,
                KeyCode::Right => execute!(stdout, MoveRight(1))?,
                KeyCode::Left => execute!(stdout, MoveLeft(1))?,
                KeyCode::Backspace => {
                    execute!(stdout, MoveLeft(1))?;
                    write!(stdout, " ")?;
                    stdout.flush()?;
                    execute!(stdout, MoveLeft(1))?;
                }
                KeyCode::Char(ch) => {
                    if let Some(digit) = ch.to_digit(10) {
                        write!(stdout, "{}", DIGITS[digit as usize])?;
                        stdout.flush()?;
                    } else {
                        write!(stdout, "{}", ch)?;
                        stdout.flush()?;
                    }
                }
                _ => (),
            };
        }
    }

    finish()
}
