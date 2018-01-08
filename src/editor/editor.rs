//! The Synless tree editor itself (the terminal application).

use coord::*;
use tree::{Tree, Cursor};
use style::{Color, Style, ColorTheme};
use terminal::{Terminal, Key};
use terminal::Event::{MouseEvent, KeyEvent};
use render::render;
use language::Language;
use editor::keymap::KeyMap;
use editor::keymap::Action;
use editor::command::Command;
use editor::command::Command::*;


const CENTERLINE: f32 = 0.3;

const RAINBOW: [Color; 6] = [
    Color::Red, Color::Yellow, Color::Green,
    Color::Cyan, Color::Blue, Color::Magenta];

pub struct Editor<'t, 'l : 't> {
    terminal: Terminal,
    language: &'l Language,
    keymap:   KeyMap,
    cursor:   Cursor<'t, 'l>,
    keygroup: Option<String>
}

impl<'t, 'l> Editor<'t, 'l> {
    /// Construct a new Synless tree editor.
    pub fn new(language: &'l Language,
               keymap: KeyMap,
               theme: ColorTheme,
               tree: &'t mut Tree<'l>)
               -> Editor<'t, 'l>
    {
        Editor{
            terminal: Terminal::new(theme),
            language: language,
            keymap:   keymap,
            cursor:   Cursor::new(tree),
            keygroup: None
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

    fn rainbow(&mut self) {
        for i in 0..7 {
            let color = RAINBOW[i % 6];
            let pos = Pos{ row: 41 as Row, col: 5 * i as Col };
            self.terminal.print_str("Color", pos, Style::color(color));
            let pos = Pos{ row: 42 as Row, col: 5 * i as Col };
            self.terminal.print_str("Color", pos, Style::reverse_color(color));
        }
    }
    
    fn display(&mut self) {
        render(self.cursor.as_ref(),
               self.cursor.char_index(),
               &mut self.terminal,
               CENTERLINE);
//        self.background();
        self.rainbow();
        self.terminal.present();
    }

    fn press_mouse(&mut self, x: i32, y: i32) {
        self.terminal.simple_print(&format!("{:?} {:?}", x, y),
                                   Pos{ row: 40, col: 1 });
    }

    fn perform(&mut self, command: &Command) {
        debug!("Command: {}", command);
        match command {
            // Tree Navigation
            &Right => { self.cursor.right(); },
            &Left  => { self.cursor.left(); },
            &Up    => { self.cursor.up(); },
            &Down  => { self.cursor.down(); },
            // Text Navigation
            &RightChar => { self.cursor.right_char(); },
            &LeftChar  => { self.cursor.left_char(); },
            // Modes
            &EnterText => { self.cursor.enter_text(); },
            &ExitText  => { self.cursor.exit_text(); },
            // Tree Editing
            &AddChild => { self.cursor.add_child(); },
            &DeleteTree => { self.cursor.delete_tree(); },
            &ReplaceTree(ref name) => {
                let con = self.language.lookup_name(name);
                self.cursor.replace_tree(con);
            }
            // Text Editing
            &InsertChar(ch) => { self.cursor.insert_char(ch); },
            &DeleteChar => { self.cursor.delete_char(); }
        }
    }

    fn get_keymap(&self) -> &KeyMap {
        match self.keygroup {
            None => &self.keymap,
            Some(ref group) => self.keymap.lookup_keygroup(group)
        }
    }

    fn lookup_key(&self, key: Key) -> Option<Action> {
        let keymap = self.get_keymap();
        keymap.lookup(key, self.cursor.mode())
    }

    fn press_key(&mut self, key: Key) {
        if let Some(action) = self.lookup_key(key) {
            self.keygroup = None;
            match action {
                Action::Command(cmd) => self.perform(&cmd),
                Action::KeyGroup(group) => {
                    debug!("Entering key group {}", group);
                    self.keygroup = Some(group);
                }
            }
        }
        self.terminal.simple_print(&format!("{:?}", key),
                                   Pos{ row: 40, col: 1 });
        self.terminal.simple_print(&format!("{:?}", self.cursor.path()),
                                   Pos{ row: 40, col: 20 });
        self.terminal.simple_print(&format!("{}", self.cursor.mode()),
                                   Pos{ row: 40, col: 40});
        if let &Some(ref group) = &self.keygroup {
            self.terminal.simple_print(group,
                                       Pos{ row: 40, col: 50});
        }
    }
}


/*
pub type Action<S> = Box<Fn(&mut S) -> ()>;

pub type KeyMap<S> = HashMap<Key, Action<S>>;
*/
