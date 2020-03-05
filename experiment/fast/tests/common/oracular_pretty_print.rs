//! Slow, but definitely correct, pretty printer.

use fast::Notation;
use Notation::*;

pub fn oracular_pretty_print(notation: &Notation, width: usize) -> Vec<(usize, String)> {
    pp(notation)
        .render(width)
        .expect("oracular_pp: impossible notation")
}

fn pp(notation: &Notation) -> Doc {
    match notation {
        Literal(text) => Doc::new_string(text),
        Nest(indent, notation) => {
            let mut bottom = pp(notation);
            bottom.indent(*indent);
            Doc::vert(Doc::new_string(""), bottom)
        }
        Flat(notation) => pp(notation).flat(),
        Concat(left, right) => Doc::concat(pp(left), pp(right)),
        Align(notation) => match pp(notation) {
            Doc::Impossible => Doc::Impossible,
            doc @ Doc::Line(_, _) => doc,
            doc => Doc::align(doc),
        },
        Choice(opt1, opt2) => Doc::choice(pp(opt1), pp(opt2)),
    }
}

/// A document, half-rendered by the Oracle.
///
/// INVARIANTS:
/// - The first line has no indent.
/// - `Impossible` is only ever at the root (it's never inside an Align, Vert, or Choice)
/// - One `Align` is never inside another.
#[derive(Debug, Clone)]
enum Doc {
    /// Attempted to flatten a document containing a required newline.
    Impossible,
    /// A line of text: `String` indented to the right by `usize` spaces.
    Line(usize, String),
    /// An aligned set of lines. When indented, they move as a group.
    Align(Box<Doc>),
    /// Vertical concatentation: left + newline + right.
    Vert(Box<Doc>, Box<Doc>),
    /// Choice: use left if it fits, otherwise use right.
    Choice(Box<Doc>, Box<Doc>),
}

impl Doc {
    fn new_string(s: &str) -> Doc {
        Doc::Line(0, s.to_string())
    }

    fn is_impossible(&self) -> bool {
        match self {
            Doc::Impossible => true,
            _ => false,
        }
    }

    fn align(doc: Doc) -> Doc {
        if doc.is_impossible() {
            return Doc::Impossible;
        }
        Doc::Align(Box::new(doc.remove_aligns()))
    }

    fn vert(top: Doc, bottom: Doc) -> Doc {
        if top.is_impossible() || bottom.is_impossible() {
            return Doc::Impossible;
        }
        Doc::Vert(Box::new(top), Box::new(bottom))
    }

    fn choice(opt1: Doc, opt2: Doc) -> Doc {
        match (opt1, opt2) {
            (Doc::Impossible, Doc::Impossible) => Doc::Impossible,
            (Doc::Impossible, doc) => doc,
            (doc, Doc::Impossible) => doc,
            (doc1, doc2) => Doc::Choice(Box::new(doc1), Box::new(doc2)),
        }
    }

    fn concat(left: Doc, right: Doc) -> Doc {
        let left = left;
        match (left, right) {
            (Doc::Impossible, _) | (_, Doc::Impossible) => Doc::Impossible,
            (Doc::Line(i, s), mut right) => {
                right.prepend(i, &s);
                right
            }
            (mut left, Doc::Line(_, s)) => {
                left.postpend(&s);
                left
            }
            (Doc::Vert(top, bottom), right) => Doc::vert(*top, Doc::concat(*bottom, right)),
            (left, Doc::Vert(top, bottom)) => Doc::vert(Doc::concat(left, *top), *bottom),
            // Remaining cases involve (align|choice)+(align|choice)
            (_, _) => panic!("oracular_pp: too choosy"),
        }
    }

    /// Delete all `Align`s inside this Doc.
    fn remove_aligns(self) -> Doc {
        match self {
            Doc::Impossible | Doc::Line(_, _) => self,
            Doc::Align(doc) => doc.remove_aligns(),
            Doc::Vert(top, bottom) => Doc::Vert(
                Box::new(top.remove_aligns()),
                Box::new(bottom.remove_aligns()),
            ),
            Doc::Choice(opt1, opt2) => Doc::Choice(
                Box::new(opt1.remove_aligns()),
                Box::new(opt2.remove_aligns()),
            ),
        }
    }

    /// Prepend text (without newlines) onto the beginning of the first line of
    /// the Doc. `indent` is the indentation level of that text, as a number of
    /// spaces.
    fn prepend(&mut self, indent: usize, text: &str) {
        match self {
            Doc::Impossible => (),
            Doc::Line(i, line) => {
                assert_eq!(*i, 0);
                *i = indent;
                *line = format!("{}{}", text, line);
            }
            Doc::Align(doc) => {
                doc.indent_all_but_first(indent + text.chars().count());
                doc.prepend(indent, text);
            }
            Doc::Vert(top, _) => top.prepend(indent, text),
            Doc::Choice(opt1, opt2) => {
                opt1.prepend(indent, text);
                opt2.prepend(indent, text);
            }
        }
    }

    /// Append text (without newlines) onto the end of the last line of the Doc.
    fn postpend(&mut self, text: &str) {
        match self {
            Doc::Impossible => (),
            Doc::Line(_, line) => line.push_str(text),
            Doc::Align(doc) => doc.postpend(text),
            Doc::Vert(_, bottom) => bottom.postpend(text),
            Doc::Choice(opt1, opt2) => {
                opt1.postpend(text);
                opt2.postpend(text);
            }
        }
    }

    /// Shift every line but the first to the right by `indent` spaces.
    fn indent_all_but_first(&mut self, indent: usize) {
        match self {
            Doc::Impossible => (),
            Doc::Line(_, _) => (),
            Doc::Align(doc) => doc.indent_all_but_first(indent),
            Doc::Vert(top, bottom) => {
                top.indent_all_but_first(indent);
                bottom.indent(indent);
            }
            Doc::Choice(opt1, opt2) => {
                opt1.indent_all_but_first(indent);
                opt2.indent_all_but_first(indent);
            }
        }
    }

    /// Shift every line to the right by `indent` spaces.
    fn indent(&mut self, indent: usize) {
        match self {
            Doc::Impossible => (),
            Doc::Line(i, _) => *i += indent,
            Doc::Align(doc) => doc.indent(indent),
            Doc::Vert(top, bottom) => {
                top.indent(indent);
                bottom.indent(indent);
            }
            Doc::Choice(opt1, opt2) => {
                opt1.indent(indent);
                opt2.indent(indent);
            }
        }
    }

    /// Flatten, eliminating any choices containing newlines.
    fn flat(self) -> Doc {
        match self {
            Doc::Impossible => Doc::Impossible,
            Doc::Line(_, _) => self,
            Doc::Align(doc) => doc.flat(),
            Doc::Vert(_, _) => Doc::Impossible,
            Doc::Choice(opt1, opt2) => Doc::choice(opt1.flat(), opt2.flat()),
        }
    }

    /// Render. Each `(usize, String)` is a line, and the `usize` is the
    /// indentation level of that line, in spaces.
    fn render(self, width: usize) -> Option<Vec<(usize, String)>> {
        match self {
            Doc::Impossible => None,
            Doc::Line(i, s) => Some(vec![(i, s)]),
            Doc::Align(doc) => doc.render(width),
            Doc::Vert(top, bottom) => {
                let mut lines = vec![];
                lines.append(&mut top.render(width)?);
                lines.append(&mut bottom.render(width)?);
                Some(lines)
            }
            Doc::Choice(opt1, opt2) => match (opt1.render(width), opt2.render(width)) {
                (Some(lines1), Some(lines2)) => {
                    if fits_width(&lines1, width) {
                        Some(lines1)
                    } else {
                        Some(lines2)
                    }
                }
                (Some(lines1), None) => Some(lines1),
                (None, Some(lines2)) => Some(lines2),
                (None, None) => None,
            },
        }
    }
}

/// Does the rendered document (`(index, text)` for each line) fit within the
/// given width?
fn fits_width(lines: &[(usize, String)], width: usize) -> bool {
    // We only check if the first and last lines fit! This is definitely poor
    // behavior. However, it is the price we pay for a fast pretty printing
    // algorithm.
    if let Some((i, s)) = lines.first() {
        if i + s.chars().count() > width {
            return false;
        }
    }
    if let Some((i, s)) = lines.last() {
        if i + s.chars().count() > width {
            return false;
        }
    }
    true
}
