use std::collections::HashMap;
use std::iter;
use termion::event::Key;

use language::{ArityType, Language, LanguageName};

use crate::keymaps::{FilterRule, TreeKmapFactory};
use crate::prog::{Prog, Value, Word};

pub fn make_node_map<'l>(lang: &Language) -> TreeKmapFactory<'l> {
    TreeKmapFactory::new(
        lang.keymap()
            .iter()
            .map(|(&ch, construct_name)| {
                (
                    Key::Char(ch),
                    FilterRule::Sort(lang.lookup_construct(construct_name).sort.clone()),
                    Prog::named(
                        construct_name,
                        &[
                            Word::Literal(Value::LangConstruct(
                                lang.name().into(),
                                construct_name.to_owned(),
                            )),
                            Word::NodeByName,
                            Word::Replace,
                        ],
                    ),
                )
            })
            .chain(iter::once((
                Key::Esc,
                FilterRule::Always,
                Prog::named(
                    "Cancel",
                    &[
                        Word::Literal(Value::Message("Cancelled node replacement!".into())),
                        Word::Echo,
                    ],
                ),
            )))
            .collect(),
    )
}

pub fn make_tree_map<'l>() -> TreeKmapFactory<'l> {
    TreeKmapFactory::new(vec![
        (Key::Char('d'), FilterRule::Always, Prog::single(Word::Cut)),
        (Key::Char('y'), FilterRule::Always, Prog::single(Word::Copy)),
        (
            Key::Char('P'),
            FilterRule::Always,
            Prog::single(Word::PasteSwap),
        ),
        (
            Key::Char('p'),
            FilterRule::Always,
            Prog::named("PasteReplace", &[Word::PasteSwap, Word::PopClipboard]),
        ),
        (Key::Char('u'), FilterRule::Always, Prog::single(Word::Undo)),
        (Key::Ctrl('r'), FilterRule::Always, Prog::single(Word::Redo)),
        (Key::Right, FilterRule::Always, Prog::single(Word::Right)),
        (Key::Left, FilterRule::Always, Prog::single(Word::Left)),
        (Key::Up, FilterRule::Always, Prog::single(Word::Parent)),
        (
            Key::Backspace,
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::Remove),
        ),
        (
            Key::Char('x'),
            FilterRule::Always,
            Prog::single(Word::Clear),
        ),
        (
            Key::Down,
            FilterRule::Always,
            Prog::named("Child", &[Word::Literal(Value::Usize(0)), Word::Child]),
        ),
        (
            Key::Char('i'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHoleAfter),
        ),
        (
            Key::Char('I'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHoleBefore),
        ),
        (
            Key::Char('o'),
            FilterRule::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHolePostpend),
        ),
        (
            Key::Char('O'),
            FilterRule::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHolePrepend),
        ),
        (
            Key::Char('r'),
            FilterRule::Always,
            Prog::named(
                "Replace",
                &[
                    Word::Replace.quote(),
                    Word::Literal(Value::MenuName("node".into())),
                    Word::ActivateMenu,
                ],
            ),
        ),
        (
            Key::Char(' '),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::named(
                "SpeedBoolMode",
                &[
                    Word::Literal(Value::ModeName("speed_bool".into())),
                    Word::PushMode,
                ],
            ),
        ),
        (
            Key::Char('m'),
            FilterRule::Always,
            Prog::named(
                "Mark",
                &[Word::Literal(Value::Char('m')), Word::SetBookmark],
            ),
        ),
        (
            Key::Char('\''),
            FilterRule::Always,
            Prog::named(
                "GotoMark",
                &[Word::Literal(Value::Char('m')), Word::GotoBookmark],
            ),
        ),
    ])
}

pub fn make_speed_bool_map<'l>() -> TreeKmapFactory<'l> {
    let lang: LanguageName = "json".into();
    TreeKmapFactory::new(vec![
        (
            Key::Char('t'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::named(
                "True",
                &[
                    Word::Literal(Value::LangConstruct(lang.clone(), "true".into())),
                    Word::InsertHoleAfter,
                    Word::NodeByName,
                    Word::Replace,
                ],
            ),
        ),
        (
            Key::Char('f'),
            FilterRule::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::named(
                "False",
                &[
                    Word::Literal(Value::LangConstruct(lang, "false".into())),
                    Word::InsertHoleAfter,
                    Word::NodeByName,
                    Word::Replace,
                ],
            ),
        ),
        (
            Key::Esc,
            FilterRule::Always,
            Prog::named("Exit", &[Word::PopMode]),
        ),
    ])
}

pub fn make_text_map<'l>() -> HashMap<Key, Prog<'l>> {
    let bindings = vec![
        (Key::Esc, Prog::single(Word::TreeMode)),
        (Key::Up, Prog::single(Word::TreeMode)),
        (Key::Left, Prog::single(Word::TextLeft)),
        (Key::Right, Prog::single(Word::TextRight)),
        (Key::Backspace, Prog::single(Word::DeleteCharBackward)),
        (Key::Delete, Prog::single(Word::DeleteCharForward)),
    ];

    bindings.into_iter().collect()
}
