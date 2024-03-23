use crossterm::cursor::MoveLeft;
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use std::io;

const HELP: &str = r#"Testing the crossterm crate:
 - ESCAPE to quit!
 - Any other key to display its event.
"#;

fn main() -> Result<(), io::Error> {
    println!("{}", HELP);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
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
        }

        if event == Event::Key(KeyCode::Esc.into()) {
            break;
        }
    }

    stdout.execute(MoveLeft(500))?;
    disable_raw_mode()
}
