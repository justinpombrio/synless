//! An editable language.

use std::collections::HashMap;

use style::{Style, Color, Emph, Shade};
use syntax::{Syntax, Repeat, Construct, Arity};
use syntax::{empty, literal, text, child, flush, if_empty_text, repeat, star};
#[cfg(test)]
use doc::Tree;

/// The syntax and whatnot for a language.
/// Right now it just has a mapping from key to syntactic construct.
/// Eventually, the language, which describes syntactic constructs,
/// should be separated from the notation that describes how to
/// display it.
pub struct Language {
    pub(crate) keymap: HashMap<char, Construct>
}

fn punct(s: &str) -> Syntax {
    literal(s, Style::color(Color::Yellow))
}

fn word(s: &str) -> Syntax {
    literal(s, Style::color(Color::Green))
}

fn txt() -> Syntax {
    text(Style::new(Color::Blue, Emph::Underline, Shade::black(), false))
}

impl Language {

    /// An example language for testing.
    pub fn example_language() -> Language {
        let mut keymap = HashMap::new();
        keymap.insert('p', {
            let syn =
                child(0) + punct(" + ") + child(1)
                | flush(child(0)) + punct("+ ") + child(1);
            Construct::new("plus", Arity::fixed(2), syn)
        });
        keymap.insert('a', {
            let syn = repeat(Repeat{
                empty:  empty(),
                lone:   star(),
                first:  star() + punct(", "),
                middle: star() + punct(", "),
                last:   star()
            })| repeat(Repeat{
                empty:  empty(),
                lone:   star(),
                first:  flush(star() + punct(",")),
                middle: flush(star() + punct(",")),
                last:   star()
            });
            Construct::new("args", Arity::extendable(0), syn)
        });
        keymap.insert('l', {
            let syn = repeat(Repeat{
                empty:  punct("[]"),
                lone:   punct("[") + star() + punct("]"),
                first:  punct("[") + star() + punct(", "),
                middle: star() + punct(", "),
                last:   star() + punct("]")
            })| repeat(Repeat{
                empty:  punct("[]"),
                lone:   punct("[") + star() + punct("]"),
                first:  flush(star() + punct(",")),
                middle: flush(star() + punct(",")),
                last:   star() + punct("]")
            })| repeat(Repeat{
                empty:  punct("[]"),
                lone:   punct("[") + star() + punct("]"),
                first:  punct("[")
                    + (star() + punct(", ") | flush(star() + punct(","))),
                middle: star() + punct(", ") | flush(star() + punct(",")),
                last:   star() + punct("]")
            });
            Construct::new("list", Arity::extendable(0), syn)
        });
        keymap.insert('f', {
            let syn =
                word("func ") + child(0)
                + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
              | flush(word("func ") + child(0) + punct("(") + child(1) + punct(") {"))
                + flush(word("  ") + child(2))
                + punct("}")
              | flush(word("func ") + child(0) + punct("("))
                + flush(word("  ") + child(1) + punct(")"))
                + flush(punct("{"))
                + flush(word("  ") + child(2))
                + punct("}");
            Construct::new("func", Arity::fixed(3), syn)
        });
        keymap.insert('i', {
            let syn = if_empty_text(txt() + punct("·"), txt());
            Construct::new("id", Arity::text(), syn)
        });
        keymap.insert('s', {
            let syn = punct("'") + txt() + punct("'");
            Construct::new("string", Arity::text(), syn)
        });
        Language{
            keymap: keymap
        }
    }
}


#[cfg(test)]
pub(crate) fn make_example_tree<'l>(lang: &'l Language, tweak: bool) -> Tree<'l> {
    let con_func = &lang.keymap[&'f'];
    let con_id   = &lang.keymap[&'i'];
    let con_str  = &lang.keymap[&'s'];
    let con_arg  = &lang.keymap[&'a'];
    let con_plus = &lang.keymap[&'p'];

    let foo = Tree::new_text(con_id, "foo");
    let abc = Tree::new_text(con_id, "abc");
    let def = Tree::new_text(con_id, "def");
    let args = Tree::new_forest(con_arg, vec!(abc, def));
    let abcdef1 = Tree::new_text(con_str, "abcdef");
    let abcdef2 = if tweak {
        Tree::new_text(con_str, "abc")
    } else {
        Tree::new_text(con_str, "abcdef")
    };
    let body = Tree::new_forest(con_plus, vec!(abcdef1, abcdef2));
    Tree::new_forest(con_func, vec!(foo, args, body))
}
