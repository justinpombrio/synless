use std::{io, thread, time};

use termion::event::Key;

use frontends::{Event, Frontend, Terminal};
use pretty::{Color, ColorTheme, Pos, PrettyScreen, Row, Style};

fn main() -> Result<(), io::Error> {
    let mut demo = TermDemo::new(20)?;
    demo.run()?;
    drop(demo);
    println!("Exited alternate screen. Your cursor should be visible again.");
    Ok(())
}

/// Demonstrate basic features of the terminal frontend.
struct TermDemo {
    /// How many events to handle before exiting.
    num_events: usize,
    /// The current line to print text to.
    line: Row,
    /// The underlying terminal.
    term: Terminal,
}

impl TermDemo {
    fn new(num_events: usize) -> Result<Self, io::Error> {
        Ok(Self {
            num_events,
            line: 0,
            term: Terminal::new(ColorTheme::default_dark())?,
        })
    }

    /// Respond to some key/mouse events, then quit.
    fn run(&mut self) -> Result<(), io::Error> {
        self.intro()?;
        for _ in 0..self.num_events {
            if !self.handle_event()? {
                // We quit early.
                return Ok(());
            }
        }

        self.println(
            &format!("Handled all {} events, goodbye!", self.num_events),
            Style::reverse_color(Color::Base09),
        )?;
        thread::sleep(time::Duration::from_secs(1));

        Ok(())
    }

    /// Print an intro message explaining how the demo will work.
    fn intro(&mut self) -> Result<(), io::Error> {
        self.println(
            "This is a demo of terminal frontend features.",
            Style::plain(),
        )?;
        self.println("Click to paint!", Style::color(Color::Base08))?;
        self.println(
            "Type q to quit, c to clear screen, or s to print size.",
            Style::color(Color::Base0A),
        )?;
        self.println(
            "Or type something else to print its event below.",
            Style::color(Color::Base0B),
        )?;
        self.println(
            &format!(
                "The demo will end after {} keypresses or clicks.",
                self.num_events
            ),
            Style::color(Color::Base0D),
        )
    }

    /// Wait for an event, then handle it. Return false if we should quit.
    fn handle_event(&mut self) -> Result<bool, io::Error> {
        match self.term.next_event() {
            Some(Ok(Event::MouseEvent(pos))) => {
                self.paint(pos)?;
            }
            Some(Ok(Event::KeyEvent(Key::Char('q')))) => {
                self.println(
                    &format!("Quitting, goodbye!"),
                    Style::reverse_color(Color::Base09),
                )?;
                thread::sleep(time::Duration::from_secs(1));
                return Ok(false);
            }
            Some(Ok(Event::KeyEvent(Key::Char('c')))) => {
                self.clear()?;
            }
            Some(Ok(Event::KeyEvent(Key::Char('s')))) => {
                let size = self.term.size()?;
                self.println(
                    &format!("size: ({},{})", size.row, size.col),
                    Style::reverse_color(Color::Base0E),
                )?;
            }
            Some(Ok(Event::KeyEvent(Key::Char(c)))) => {
                self.println(
                    &format!("got character: {}", c),
                    Style::reverse_color(Color::Base0B),
                )?;
            }
            Some(Ok(Event::KeyEvent(ev))) => {
                self.println(
                    &format!("got other key event: {:?}", ev),
                    Style::reverse_color(Color::Base0C),
                )?;
            }
            Some(Err(err)) => {
                self.println(
                    &format!("got error: {:?}", err),
                    Style::reverse_color(Color::Base0E),
                )?;
            }
            None => {
                self.println("got no event!", Style::reverse_color(Color::Base08))?;
            }
        }
        Ok(true)
    }

    /// Clear everything that was printed to the terminal.
    fn clear(&mut self) -> Result<(), io::Error> {
        self.term.clear()?;
        self.term.show()?;
        self.line = 0;
        Ok(())
    }

    /// Draw a blue '!' at the given position.
    fn paint(&mut self, pos: Pos) -> Result<(), io::Error> {
        self.term
            .print_char('!', pos, Style::reverse_color(Color::Base0D))?;
        self.term.show()
    }

    /// Print the text on the next line.
    fn println(&mut self, text: &str, style: Style) -> Result<(), io::Error> {
        self.term.print(
            Pos {
                col: 0,
                row: self.line,
            },
            text,
            style,
        )?;
        self.line += 1;
        self.term.show()
    }
}
