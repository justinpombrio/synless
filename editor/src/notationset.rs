use lazy_static::lazy_static;
use std::collections::HashMap;

use language::{ConstructName, Language, LanguageName};
use pretty::{child, literal, Notation, Style};
use utility::GrowOnlyMap;

pub struct NotationSet {
    name: LanguageName,
    notations: HashMap<ConstructName, Notation>,
}

pub type NotationSets = GrowOnlyMap<LanguageName, NotationSet>;

lazy_static! {
    /// Notations for built-in constructs that can appear in any document.
    pub static ref BUILTIN_NOTATIONS: HashMap<ConstructName, Notation> =
        vec![
            ("hole".into(), literal("?", Style::plain())),
            ("root".into(), child(0)),
        ].into_iter().collect();
}

impl NotationSet {
    // TODO: validate against language
    pub fn new(language: &Language, notations: Vec<(ConstructName, Notation)>) -> NotationSet {
        let mut map = HashMap::new();
        for (construct, notation) in notations {
            map.insert(construct, notation);
        }
        NotationSet {
            name: language.name().to_owned(),
            notations: map,
        }
    }

    pub fn hole() -> &'static Notation {
        BUILTIN_NOTATIONS
            .get(&"hole".into())
            .expect("no builtin 'hole' notation found")
    }

    pub fn lookup(&self, construct: &ConstructName) -> &Notation {
        match self.notations.get(construct) {
            None => match BUILTIN_NOTATIONS.get(construct) {
                None => panic!(
                    "Construct {:?} not found in notation set for {:?}",
                    construct, self.name
                ),
                Some(notation) => notation,
            },

            Some(notation) => notation,
        }
    }
}

#[cfg(test)]
mod example {
    use super::*;
    use language::{Arity, Construct, Language};
    use pretty::*;

    fn punct(s: &str) -> Notation {
        literal(s, Style::color(Color::Base0A))
    }

    fn word(s: &str) -> Notation {
        literal(s, Style::color(Color::Base0B))
    }

    fn txt() -> Notation {
        text(Style {
            color: Color::Base0D,
            emph: Emph::underlined(),
            reversed: false,
        })
    }

    /// An example language for testing.
    pub fn example_language() -> (Language, NotationSet) {
        let mut language = Language::new("TestLang".into());

        let arity = Arity::Fixed(vec!["Expr".into(), "Expr".into()]);
        let construct = Construct::new("plus", "Expr", arity, Some('p'));
        language.add(construct);
        let plus_notation =
            (child(0) + punct(" + ") + child(1)) | child(0) ^ (punct("+ ") + child(1));

        let notation = NotationSet::new(&language, vec![("plus".into(), plus_notation)]);
        (language, notation)
        /*
        let syn = repeat(Repeat{
            empty:  empty(),
            lone:   star(),
            first:  star() + punct(", "),
            middle: star() + punct(", "),
            last:   star()
        }) | repeat(Repeat{
            empty:  empty(),
            lone:   star(),
            first:  flush(star() + punct(",")),
            middle: flush(star() + punct(",")),
            last:   star()
        });
        lang.add('a', Construct::new("args", Arity::extendable(0), syn));

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
        lang.add('l', Construct::new("list", Arity::extendable(0), syn));

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
        lang.add('f', Construct::new("func", Arity::fixed(3), syn));

        let syn = if_empty_text(txt() + punct("·"), txt());
        lang.add('i', Construct::new("iden", Arity::text(), syn));

        let syn = punct("'") + txt() + punct("'");
        lang.add('s', Construct::new("strn", Arity::text(), syn));
         */
    }

    /*
        pub fn example_tree<'l>(lang: &'l Language, tweak: bool) -> Tree<'l> {
            let con_func = lang.lookup_name("func");
            let con_id   = lang.lookup_name("iden");
            let con_str  = lang.lookup_name("strn");
            let con_arg  = lang.lookup_name("args");
            let con_plus = lang.lookup_name("plus");

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
    */
}
