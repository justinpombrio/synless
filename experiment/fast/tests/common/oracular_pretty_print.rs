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
        Align(notation) => Doc::Align(Box::new(pp(notation))),
        Choice(opt1, opt2) => Doc::choice(pp(opt1), pp(opt2)),
    }
}

enum Doc {
    Impossible,
    Line(usize, String),
    Align(Box<Doc>),
    Vert(Box<Doc>, Box<Doc>),
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
                right.prepend(i, &s);
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

    fn prepend(&mut self, indent: usize, text: &str) {
        match self {
            Doc::Impossible => (),
            Doc::Line(i, line) => {
                *i = indent;
                *line = format!("{}{}", text, line);
            }
            Doc::Align(doc) => {
                doc.align(text.chars().count());
                doc.prepend(indent, text);
            }
            Doc::Vert(top, _) => top.prepend(indent, text),
            Doc::Choice(opt1, opt2) => {
                opt1.prepend(indent, text);
                opt2.prepend(indent, text);
            }
        }
    }

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

    fn align(&mut self, indent: usize) {
        match self {
            Doc::Impossible => (),
            Doc::Line(_, _) => (),
            Doc::Align(doc) => doc.align(indent),
            Doc::Vert(top, bottom) => {
                top.align(indent);
                bottom.indent(indent);
            }
            Doc::Choice(opt1, opt2) => {
                opt1.align(indent);
                opt2.align(indent);
            }
        }
    }

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

    fn flat(self) -> Doc {
        match self {
            Doc::Impossible => Doc::Impossible,
            Doc::Line(_, _) => self,
            Doc::Align(doc) => doc.flat(),
            Doc::Vert(_, _) => Doc::Impossible,
            Doc::Choice(opt1, opt2) => Doc::choice(opt1.flat(), opt2.flat()),
        }
    }

    fn render(self, width: usize) -> Option<Vec<(usize, String)>> {
        match self {
            Doc::Impossible => None,
            Doc::Line(i, s) => Some(vec![(i, s)]),
            Doc::Align(doc) => doc.render(width),
            Doc::Vert(top, bottom) => {
                let mut lines = top.render(width)?;
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

fn fits_width(lines: &[(usize, String)], width: usize) -> bool {
    for (i, s) in lines {
        if i + s.chars().count() > width {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_literal() {
        let note = Notation::literal("foo");
        let lines = oracular_pretty_print(&note, 5);
        assert_eq!(lines, vec![(0, "foo".into())]);
    }

    #[test]
    fn test_oracle_concat() {
        let note = Notation::concat(Notation::literal("foo"), Notation::literal("bar"));
        let lines = oracular_pretty_print(&note, 5);
        assert_eq!(lines, vec![(0, "foobar".into())]);
    }
}
