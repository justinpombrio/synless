use crossterm::cursor::MoveLeft;
use crossterm::event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use std::io;

// ## Some weird terminal behavior
//
// C-shift -- doesn't work, there's no shift modifier and the key comes out lowercase.
//
// A-[  --  ignores the next character!
//
// Control has weird behavior on many keys:
//
//     C-3  ==  Esc
//     C-8  ==  Backspace
//     C-i  ==  Tab
//     C-m  ==  Enter
//     C-'  ==  \
//     C-[  ==  Esc
//     C-2  ==  C-' '
//     C-`  ==  C-' '
//     C-\  ==  C-4
//     C-]  ==  C-5
//     C-/  ==  C-7
//
// The control modifier is lost on the following characters:
//
//     = 1 9 ; , .
//
// The shift modifier only appears on letters and BackTab.

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
                "{:8}{:12}{:30}{:?}",
                format!("{:?}", key_event.kind),
                format!("{:?}", key_event.code),
                format!("{:?}", key_event.modifiers),
                key_event.state,
            );
            stdout.execute(MoveLeft(500))?;
        } else if let Event::Mouse(mouse_event) = event {
            println!(
                "{:15}{:8}{:?}",
                format!("{:?}", mouse_event.kind),
                format!("{:?},{:?}", mouse_event.column, mouse_event.row),
                mouse_event.modifiers,
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
