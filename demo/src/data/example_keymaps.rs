use frontends::Key;
use std::iter;

use language::{ArityType, Language, LanguageName};

use crate::keymaps::{FilterRule, TextKeymap, TreeKeymap};
use crate::prog::{Prog, Value, Word};

/// Construct a keymap used for selecting a type of node, eg. for the Replace
/// command.
pub fn make_node_map<'l>(lang: &Language) -> TreeKeymap<'l> {
    TreeKeymap::new(
        lang.keymap()
            .iter()
            .map(|(&ch, construct_name)| {
                (
                    Key::Char(ch),
                    FilterRule::Sort(lang.lookup_construct(construct_name).sort.clone()),
                    Prog::new(&[
                        Word::Literal(Value::LangConstruct(
                            lang.name().into(),
                            construct_name.to_owned(),
                        )),
                        Word::NodeByName,
                        Word::Replace,
                    ])
                    .with_name(construct_name),
                )
            })
            .chain(iter::once((
                Key::Esc,
                FilterRule::Always,
                Prog::new(&[
                    Word::Literal(Value::String("Cancelled node replacement!".into())),
                    Word::Print,
                ])
                .with_name("Cancel"),
            )))
            .collect(),
    )
}

/// Construct a keymap used for navigating and editing a document that's in tree
/// mode.
pub fn make_tree_map<'l>() -> TreeKeymap<'l> {
    TreeKeymap::new(vec![
        (
            Key::Char('d'),
            FilterRule::Always,
            Prog::new_single(Word::Cut),
        ),
        (
            Key::Char('y'),
            FilterRule::Always,
            Prog::new_single(Word::Copy),
        ),
        (
            Key::Char('P'),
            FilterRule::Always,
            Prog::new_single(Word::PasteSwap),
        ),
        (
            Key::Char('p'),
            FilterRule::Always,
            Prog::new(&[Word::PasteSwap, Word::PopClipboard]).with_name("PasteReplace"),
        ),
        (
            Key::Char('u'),
            FilterRule::Always,
            Prog::new_single(Word::Undo),
        ),
        (
            Key::Ctrl('r'),
            FilterRule::Always,
            Prog::new_single(Word::Redo),
        ),
        (
            Key::Right,
            FilterRule::Always,
            Prog::new_single(Word::Right),
        ),
        (Key::Left, FilterRule::Always, Prog::new_single(Word::Left)),
        (Key::Up, FilterRule::Always, Prog::new_single(Word::Parent)),
        (
            Key::Backspace,
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new_single(Word::Remove),
        ),
        (
            Key::Char('x'),
            FilterRule::Always,
            Prog::new_single(Word::Clear),
        ),
        (
            Key::Down,
            FilterRule::Always,
            Prog::new(&[Word::Literal(Value::Usize(0)), Word::Child]).with_name("Child"),
        ),
        (
            Key::Char('i'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new_single(Word::InsertHoleAfter),
        ),
        (
            Key::Char('I'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new_single(Word::InsertHoleBefore),
        ),
        (
            Key::Char('o'),
            FilterRule::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new_single(Word::InsertHolePostpend),
        ),
        (
            Key::Char('O'),
            FilterRule::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new_single(Word::InsertHolePrepend),
        ),
        (
            Key::Char('r'),
            FilterRule::Always,
            Prog::new(&[
                Word::Literal(Prog::from(Word::Replace).quote()),
                Word::Literal(Value::MenuName("node".into())),
                Word::ActivateMenu,
            ])
            .with_name("Replace"),
        ),
        (
            Key::Char(' '),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new(&[
                Word::Literal(Value::ModeName("speed_bool".into())),
                Word::PushMode,
            ])
            .with_name("SpeedBoolMode"),
        ),
        (
            Key::Char('m'),
            FilterRule::Always,
            Prog::new(&[Word::Literal(Value::Char('m')), Word::SetBookmark]).with_name("Mark"),
        ),
        (
            Key::Char('\''),
            FilterRule::Always,
            Prog::new(&[Word::Literal(Value::Char('m')), Word::GotoBookmark]).with_name("GotoMark"),
        ),
    ])
}

/// Construct a keymap for a silly `speed-bool` mode that demonstrates
/// persistent mode-style keymaps. It lets you insert new `true` or `false`
/// nodes into a list by repeatedly pressing a single key.
pub fn make_speed_bool_map<'l>() -> TreeKeymap<'l> {
    let lang: LanguageName = "json".into();
    TreeKeymap::new(vec![
        (
            Key::Char('t'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new(&[
                Word::Literal(Value::LangConstruct(lang.clone(), "true".into())),
                Word::InsertHoleAfter,
                Word::NodeByName,
                Word::Replace,
            ])
            .with_name("True"),
        ),
        (
            Key::Char('f'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::new(&[
                Word::Literal(Value::LangConstruct(lang, "false".into())),
                Word::InsertHoleAfter,
                Word::NodeByName,
                Word::Replace,
            ])
            .with_name("False"),
        ),
        (
            Key::Esc,
            FilterRule::Always,
            Prog::new(&[Word::PopMode]).with_name("Exit"),
        ),
    ])
}

/// Make a keymap used for editing texty nodes.
pub fn make_text_map<'l>() -> TextKeymap<'l> {
    let bindings = vec![
        (Key::Esc, Prog::new_single(Word::TreeMode)),
        (Key::Up, Prog::new_single(Word::TreeMode)),
        (Key::Left, Prog::new_single(Word::TextLeft)),
        (Key::Right, Prog::new_single(Word::TextRight)),
        (Key::Backspace, Prog::new_single(Word::DeleteCharBackward)),
        (Key::Delete, Prog::new_single(Word::DeleteCharForward)),
    ];

    TextKeymap::new(bindings.into_iter().collect())
}
