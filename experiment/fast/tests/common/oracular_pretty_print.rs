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
        Concat(left, right) => pp(left).concat(pp(right)),
        Align(notation) => match pp(notation) {
            doc @ Doc::Line(_, _) => doc,
            doc => Doc::Align(Box::new(doc)),
        },
        Choice(opt1, opt2) => Doc::choice(pp(opt1), pp(opt2)),
    }
}

/// A document, half-rendered by the Oracle.
///
/// INVARIANT: The first line has no indent.
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

    fn vert(top: Doc, bottom: Doc) -> Doc {
        Doc::Vert(Box::new(top), Box::new(bottom))
    }

    fn choice(opt1: Doc, opt2: Doc) -> Doc {
        Doc::Choice(Box::new(opt1), Box::new(opt2))
    }

    fn concat(self, right: Doc) -> Doc {
        let left = self;
        match (left, right) {
            (Doc::Impossible, _) | (_, Doc::Impossible) => Doc::Impossible,
            (Doc::Line(i, s), mut right) => {
                right.prepend(i, &s, false);
                right
            }
            (mut left, Doc::Line(_, s)) => {
                left.postpend(&s);
                left
            }
            (Doc::Vert(top, bottom), right) => Doc::vert(*top, bottom.concat(right)),
            (left, Doc::Vert(top, bottom)) => Doc::vert(left.concat(*top), *bottom),
            // Remaining cases involve (align|choice)+(align|choice)
            (_, _) => panic!("oracular_pp: too choosy"),
        }
    }

    /// Prepend text (without newlines) onto the beginning of the first line of
    /// the Doc. `indent` is the indentation level of that text, as a number of
    /// spaces.
    fn prepend(&mut self, indent: usize, text: &str, in_aligned: bool) {
        match self {
            Doc::Impossible => (),
            Doc::Line(i, line) => {
                assert_eq!(*i, 0);
                *i = indent;
                *line = format!("{}{}", text, line);
            }
            Doc::Align(doc) => {
                if !in_aligned {
                    doc.indent_all_but_first(indent + text.chars().count());
                }
                doc.prepend(indent, text, true);
            }
            Doc::Vert(top, _) => top.prepend(indent, text, in_aligned),
            Doc::Choice(opt1, opt2) => {
                opt1.prepend(indent, text, in_aligned);
                opt2.prepend(indent, text, in_aligned);
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
    for (i, s) in lines {
        if i + s.chars().count() > width {
            return false;
        }
    }
    true
}
