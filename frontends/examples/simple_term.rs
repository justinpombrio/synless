use std::{thread, time};

use termion::event::Key;

use frontends::{terminal, Event, Frontend, Terminal};
use pretty::{Bound, Color, ColorTheme, Pos, PrettyWindow, Region, Row, Shade, Style};

fn main() -> Result<(), terminal::Error> {
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
    fn new(num_events: usize) -> Result<Self, terminal::Error> {
        Ok(Self {
            num_events,
            line: 0,
            term: Terminal::new(ColorTheme::default_dark())?,
        })
    }

    /// Respond to some key/mouse events, then quit.
    fn run(&mut self) -> Result<(), terminal::Error> {
        self.term.start_frame()?;
        self.intro()?;
        self.term.show_frame()?;
        for _ in 0..self.num_events {
            self.term.start_frame()?;
            let quit_early = !self.handle_event()?;
            self.term.show_frame()?;
            if quit_early {
                return Ok(());
            }
        }

        self.term.start_frame()?;
        self.println(
            &format!("Handled all {} events, goodbye!", self.num_events),
            Style::reverse_color(Color::Base09),
        )?;
        self.term.show_frame()?;
        thread::sleep(time::Duration::from_secs(1));

        Ok(())
    }

    /// Print an intro message explaining how the demo will work.
    fn intro(&mut self) -> Result<(), terminal::Error> {
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
    fn handle_event(&mut self) -> Result<bool, terminal::Error> {
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
            Some(Ok(Event::KeyEvent(Key::Char('s')))) => {
                let size = self.term.size()?;
                self.println(
                    &format!("size: ({},{})", size.row, size.col),
                    Style::reverse_color(Color::Base0E),
                )?;
            }
            Some(Ok(Event::KeyEvent(Key::Char('r')))) => {
                let region = Region {
                    pos: Pos { row: 10, col: 1 },
                    bound: Bound {
                        width: 5,
                        height: 3,
                        indent: 2,
                    },
                };
                self.term.pane()?.pretty_pane().shade(region, Shade(0))?;
            }

            Some(Ok(Event::KeyEvent(Key::Char(c)))) => {
                self.println(
                    &format!("got character: {}", c),
                    Style::color(Color::Base0C),
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

    /// Draw a blue '!' at the given position.
    fn paint(&mut self, pos: Pos) -> Result<(), terminal::Error> {
        let mut pane = self.term.pane()?;
        pane.pretty_pane()
            .print(pos, "!", Style::color(Color::Base0D))?;
        pane.pretty_pane()
            .highlight(pos, Style::color(Color::Base0C))
    }

    /// Print the text on the next line.
    fn println(&mut self, text: &str, style: Style) -> Result<(), terminal::Error> {
        self.term.pane()?.pretty_pane().print(
            Pos {
                col: 0,
                row: self.line,
            },
            text,
            style,
        )?;
        self.line += 1;
        Ok(())
    }
}
