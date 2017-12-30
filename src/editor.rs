//! The Synless tree editor itself (the terminal application).

use coord::*;
use tree::Tree;
use doc::Cursor;
use doc::Mode::*;
use style::{Color, Style};
use terminal::{Terminal, Key};
use terminal::Event::{MouseEvent, KeyEvent};
use render::render;
use language::Language;


const CENTERLINE: f32 = 0.3;


pub struct Editor<'t, 'l : 't> {
    terminal: Terminal,
    language: &'l Language,
    cursor:   Cursor<'t, 'l>
}

impl<'t, 'l> Editor<'t, 'l> {
    /// Construct a new Synless tree editor.
    pub fn new(language: &'l Language, tree: &'t mut Tree<'l>) -> Editor<'t, 'l> {
        Editor{
            terminal: Terminal::new(),
            language: language,
            cursor:   Cursor::new(tree)
        }
    }

    /// Run the editor.
    pub fn run(&mut self) {
        self.display();
        loop {
            match (&self).terminal.poll_event() {
                None => (),
                Some(MouseEvent(x, y)) => {
                    self.clear();
                    self.press_mouse(x, y);
                    self.display();
                }
                Some(KeyEvent(Key::Esc)) => {
                    break;
                }
                Some(KeyEvent(key)) => {
                    self.clear();
                    self.press_key(key);
                    self.display();
                }
            }
        }
    }

    fn clear(&mut self) {
        self.terminal.clear();
    }

    fn splat_color(&mut self, pos: Pos, color: Color) {
        self.terminal.print_str("Color", pos, Style::color(color));
    }

    fn splat_color_rev(&mut self, pos: Pos, color: Color) {
        self.terminal.print_str("Color", pos, Style::reverse_color(color));
    }

    fn rainbow(&mut self) {
        self.splat_color_rev(Pos{ row: 41, col: 0  }, Color::Red);
        self.splat_color_rev(Pos{ row: 41, col: 5  }, Color::Yellow);
        self.splat_color_rev(Pos{ row: 41, col: 10 }, Color::Green);
        self.splat_color_rev(Pos{ row: 41, col: 15 }, Color::Cyan);
        self.splat_color_rev(Pos{ row: 41, col: 20 }, Color::Blue);
        self.splat_color_rev(Pos{ row: 41, col: 25 }, Color::Magenta);
        self.splat_color_rev(Pos{ row: 41, col: 30 }, Color::Red);
        self.splat_color(Pos{ row: 42, col: 0  }, Color::Red);
        self.splat_color(Pos{ row: 42, col: 5  }, Color::Yellow);
        self.splat_color(Pos{ row: 42, col: 10 }, Color::Green);
        self.splat_color(Pos{ row: 42, col: 15 }, Color::Cyan);
        self.splat_color(Pos{ row: 42, col: 20 }, Color::Blue);
        self.splat_color(Pos{ row: 42, col: 25 }, Color::Magenta);
        self.splat_color(Pos{ row: 42, col: 30  }, Color::Red);
    }
    
    fn display(&mut self) {
        render(self.cursor.as_ref(),
               self.cursor.char_index(),
               &mut self.terminal,
               CENTERLINE);
        self.rainbow();
        self.terminal.present();
    }

    fn press_mouse(&mut self, x: i32, y: i32) {
        self.terminal.simple_print(&format!("{:?} {:?}", x, y),
                                   Pos{ row: 40, col: 1 });
    }

    fn press_key(&mut self, key: Key) {
        match self.cursor.mode() {
            TreeMode => {
                match key {
                    Key::Up => {
                        debug!("UP");
                        self.cursor.up();
                    }
                    Key::Down => {
                        debug!("DOWN");
                        self.cursor.down();
                    }
                    Key::Left => {
                        debug!("LEFT");
                        self.cursor.left();
                    }
                    Key::Right => {
                        debug!("RIGHT");
                        self.cursor.right();
                    }
                    Key::Char('j') => {
                        debug!("ADD CHILD");
                        self.cursor.add_child();
                    }
                    Key::Backspace => {
                        debug!("DELETE TREE");
                        self.cursor.delete_tree();
                    }
                    Key::Enter => {
                        debug!("ENTER TEXT");
                        self.cursor.enter_text();
                    }
                    Key::Char(c) => {
                        match self.language.keymap.get(&c) {
                            None => (),
                            Some(construct) => {
                                debug!("INSERT {}", c);
                                self.cursor.replace_tree(construct);
                            }
                        }
                    }
                    _ => ()
                }
            }
            TextMode => {
                match key {
                    Key::Enter => {
                        debug!("EXIT TEXT");
                        self.cursor.exit_text();
                    }
                    Key::Backspace => {
                        debug!("DELETE CHAR");
                        self.cursor.delete_char();
                    }
                    Key::Left => {
                        debug!("LEFT");
                        self.cursor.left_char();
                    }
                    Key::Right => {
                        debug!("RIGHT");
                        self.cursor.right_char();
                    }
                    Key::Char(c) => {
                        self.cursor.insert_char(c);
                    }
                    _ => ()
                }                
            }
        }
        self.terminal.simple_print(&format!("{:?}", key),
                                   Pos{ row: 40, col: 1 });
        self.terminal.simple_print(&format!("{:?}", self.cursor.path()),
                                   Pos{ row: 40, col: 20 });
        self.terminal.simple_print(&format!("{}", self.cursor.mode()),
                                   Pos{ row: 40, col: 40});
    }
}


/*
pub type Action<S> = Box<Fn(&mut S) -> ()>;

pub type KeyMap<S> = HashMap<Key, Action<S>>;
*/
