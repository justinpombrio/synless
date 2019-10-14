use std::collections::HashMap;
use std::iter;
use termion::event::Key;

use language::{ArityType, Language, LanguageName};

use crate::keymap::{KmapFilter, TreeKmapFactory};
use crate::prog::{Prog, Value, Word};

pub fn make_node_map<'l>(lang: &Language) -> TreeKmapFactory<'l> {
    TreeKmapFactory::new(
        lang.keymap()
            .iter()
            .map(|(&ch, construct_name)| {
                (
                    Key::Char(ch),
                    KmapFilter::Sort(lang.lookup_construct(construct_name).sort.clone()),
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
                KmapFilter::Always,
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
        (Key::Char('d'), KmapFilter::Always, Prog::single(Word::Cut)),
        (Key::Char('y'), KmapFilter::Always, Prog::single(Word::Copy)),
        (
            Key::Char('P'),
            KmapFilter::Always,
            Prog::single(Word::PasteSwap),
        ),
        (
            Key::Char('p'),
            KmapFilter::Always,
            Prog::named("PasteReplace", &[Word::PasteSwap, Word::PopClipboard]),
        ),
        (Key::Char('u'), KmapFilter::Always, Prog::single(Word::Undo)),
        (Key::Ctrl('r'), KmapFilter::Always, Prog::single(Word::Redo)),
        (Key::Right, KmapFilter::Always, Prog::single(Word::Right)),
        (Key::Left, KmapFilter::Always, Prog::single(Word::Left)),
        (Key::Up, KmapFilter::Always, Prog::single(Word::Parent)),
        (
            Key::Backspace,
            KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::Remove),
        ),
        (
            Key::Char('x'),
            KmapFilter::Always,
            Prog::single(Word::Clear),
        ),
        (
            Key::Down,
            KmapFilter::Always,
            Prog::named("Child", &[Word::Literal(Value::Usize(0)), Word::Child]),
        ),
        (
            Key::Char('i'),
            KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHoleAfter),
        ),
        (
            Key::Char('I'),
            KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHoleBefore),
        ),
        (
            Key::Char('o'),
            KmapFilter::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHolePostpend),
        ),
        (
            Key::Char('O'),
            KmapFilter::SelfArity(vec![ArityType::Flexible, ArityType::Mixed]),
            Prog::single(Word::InsertHolePrepend),
        ),
        (
            Key::Char('r'),
            KmapFilter::Always,
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
            KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
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
            KmapFilter::Always,
            Prog::named(
                "Mark",
                &[Word::Literal(Value::Char('m')), Word::SetBookmark],
            ),
        ),
        (
            Key::Char('\''),
            KmapFilter::Always,
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
            KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
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
            KmapFilter::ParentArity(vec![ArityType::Flexible, ArityType::Mixed]),
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
            KmapFilter::Always,
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
