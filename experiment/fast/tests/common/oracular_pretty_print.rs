//! Slow, but definitely correct, pretty printer. Used for testing, by generating random notations
//! and checking if the real pretty printer produces the same output as the Oracle.

use fast::Notation;
use Notation::*;

pub fn oracular_pretty_print(notation: &Notation, width: usize) -> Vec<(usize, String)> {
    pp(notation)
        .render(width)
        .into_iter()
        .next()
        .expect("oracular_pp: impossible notation")
}

fn pp(notation: &Notation) -> Doc {
    match notation {
        Empty => Doc::new_string(""),
        Literal(text) => Doc::new_string(text),
        Newline => Doc::newline(),
        Indent(indent, notation) => pp(notation).indent(*indent),
        Flat(notation) => pp(notation).flat(),
        Concat(left, right) => Doc::concat(pp(left), pp(right)),
        Align(notation) => Doc::align(pp(notation)),
        Choice(opt1, opt2) => Doc::choice(pp(opt1), pp(opt2)),
    }
}

/// A document, half-rendered by the Oracle.
///
/// # Normalization
///
/// To simplify the Oracle's pretty-printing, `Doc`s are normalized in the
/// following ways:
///
/// 1. A `Doc::Impossible` is only ever at the root of a `Doc`. This makes it easy
///    to tell whether a `Doc` is impossible or not. For example, `Concat(x,
///    Impossible)` is normalized to `Impossible`, and `Choice(x, Impossible)`
///    is normalized to `x`. (See Choosiness Rule #1 in `validate`.)
/// 2. A `Doc::Align` must always have at least one multi-line layout option. If
///    it doesn't, then it is omitted. This is safe because it would have no
///    effect. (See Choosiness Rule #2 in `validate`.)
/// 3. One `Doc::Align` is never inside of another. If it would be, it is
///    removed. This is safe because it would have no effect. Doing this
///    simplifies the `prepend` function. (Consider `Align(Align(Nest(_, _)))`.)
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
        if !doc.may_be_multiline() {
            return doc;
        }
        Doc::Align(Box::new(doc.remove_aligns()))
    }

    fn newline() -> Doc {
        Doc::Vert(Box::new(Doc::new_string("")), Box::new(Doc::new_string("")))
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

    /// Is there at least one multi-line layout option for this Doc?
    fn may_be_multiline(&self) -> bool {
        match self {
            Doc::Impossible => false,
            Doc::Line(_, _) => false,
            Doc::Align(doc) => {
                assert!(doc.may_be_multiline());
                true
            }
            Doc::Vert(_, _) => true,
            Doc::Choice(opt1, opt2) => opt1.may_be_multiline() || opt2.may_be_multiline(),
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
                *i = indent;
                *line = format!("{}{}", text, line);
            }
            Doc::Align(doc) => {
                doc.shift_right_all_but_first(indent + text.chars().count());
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
    fn shift_right_all_but_first(&mut self, indent: usize) {
        match self {
            Doc::Impossible => (),
            Doc::Line(_, _) => (),
            Doc::Align(doc) => doc.shift_right_all_but_first(indent),
            Doc::Vert(top, bottom) => {
                top.shift_right_all_but_first(indent);
                bottom.shift_right(indent);
            }
            Doc::Choice(opt1, opt2) => {
                opt1.shift_right_all_but_first(indent);
                opt2.shift_right_all_but_first(indent);
            }
        }
    }

    /// Shift every line to the right by `indent` spaces.
    fn shift_right(&mut self, indent: usize) {
        match self {
            Doc::Impossible => (),
            Doc::Line(i, _) => *i += indent,
            Doc::Align(doc) => doc.shift_right(indent),
            Doc::Vert(top, bottom) => {
                top.shift_right(indent);
                bottom.shift_right(indent);
            }
            Doc::Choice(opt1, opt2) => {
                opt1.shift_right(indent);
                opt2.shift_right(indent);
            }
        }
    }

    fn indent(self, indent: usize) -> Doc {
        match self {
            Doc::Impossible => Doc::Impossible,
            Doc::Line(i, doc) => Doc::Line(i, doc),
            Doc::Align(doc) => Doc::Align(doc),
            Doc::Vert(top, mut bottom) => {
                bottom.shift_right(indent);
                Doc::vert(top.indent(indent), *bottom)
            }
            Doc::Choice(opt1, opt2) => Doc::choice(opt1.indent(indent), opt2.indent(indent)),
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

    /// Render. The outer vector is the set of all ways that this document
    /// _could_ be rendered. Its first element is the way that it _will_ be
    /// rendered. (The full set is required for resolving choices.)
    ///
    /// Each rendering is a list of lines. Each line is represented as a
    /// `(usize, String)` pair, where the `usize` is the indentation level as a
    /// number of spaces, and the `String` is the text of the line (not
    /// including those spaces).
    fn render(self, width: usize) -> Vec<Vec<(usize, String)>> {
        match self {
            Doc::Impossible => vec![],
            Doc::Line(i, s) => vec![vec![(i, s)]],
            Doc::Align(doc) => doc.render(width),
            Doc::Vert(top, bottom) => {
                let mut options = vec![];
                let top_opts = top.render(width);
                let bottom_opts = bottom.render(width);
                for top_opt in &top_opts {
                    for bottom_opt in &bottom_opts {
                        let mut lines = vec![];
                        lines.extend(top_opt.clone());
                        lines.extend(bottom_opt.clone());
                        options.push(lines);
                    }
                }
                options
            }
            Doc::Choice(left, right) => {
                let left_opts = left.render(width);
                let right_opts = right.render(width);

                let mut options = vec![];
                let left_fits = left_opts.iter().any(|opt| fits_width(opt, width));
                let right_impossible = right_opts.is_empty();
                if left_fits || right_impossible {
                    options.extend(left_opts);
                    options.extend(right_opts);
                } else {
                    options.extend(right_opts);
                    options.extend(left_opts);
                }
                options
            }
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
