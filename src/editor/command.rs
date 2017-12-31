use std::fmt;

use self::Command::*;


#[derive(Clone)]
pub enum Command {
    // Tree Navigation
    Right, Left, Up, Down,
    // Text Navigation
    RightChar, LeftChar,
    // Modes
    EnterText, ExitText,
    // Tree Editing
    AddChild, DeleteTree, ReplaceTree(String),
    // Text Editing
    InsertChar(char), DeleteChar
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            // Tree Navigation
            &Right => "Right",
            &Left  => "Left",
            &Up    => "Up",
            &Down  => "Down",
            // Text Navigation
            &RightChar => "RightChar",
            &LeftChar  => "LeftChar",
            // Modes
            &EnterText => "EnterText",
            &ExitText  => "ExitText",
            // Tree Editing
            &AddChild       => "AddChild",
            &DeleteTree     => "DeleteTree",
            &ReplaceTree(_) => "ReplaceTree",
            // Text Editing
            &InsertChar(_) => "InsertChar",
            &DeleteChar => "DeleteChar"
        };
        write!(f, "{}", name)
    }
}
