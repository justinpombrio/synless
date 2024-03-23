use crossterm::cursor::MoveLeft;
use crossterm::event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use std::io;

const HELP: &str = r#"Testing the crossterm crate:
 - ESCAPE to quit!
 - Any other key to display its event.
"#;

fn main() -> Result<(), io::Error> {
    println!("{}", HELP);
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    stdout.execute(EnableMouseCapture)?;
    loop {
        // Blocking read!
        let event = read()?;

        if let Event::Key(key_event) = event {
            println!(
                "{:15}{:35}{}",
                format!("{:?}", key_event.code),
                format!("{:?}", key_event.modifiers),
                format!("{:?}", key_event.state),
            );
            stdout.execute(MoveLeft(500))?;
        } else if let Event::Mouse(mouse_event) = event {
            println!(
                "{:5},{:5}{:15}{}",
                format!("{:?}", mouse_event.column),
                format!("{:?}", mouse_event.row),
                format!("{:?}", mouse_event.kind),
                format!("{:?}", mouse_event.modifiers),
            );
            stdout.execute(MoveLeft(500))?;
        }

        if event == Event::Key(KeyCode::Esc.into()) {
            break;
        }
    }

    stdout.execute(MoveLeft(500))?;
    stdout.execute(DisableMouseCapture)?;
    disable_raw_mode()
}
