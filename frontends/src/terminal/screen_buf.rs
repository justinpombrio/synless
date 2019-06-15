use pretty::{Pos, Region};
use pretty::{Shade, Style};

use super::Error;

/// Represents a screen full of characters. It buffers changes to the
/// characters, and can produce a set of instructions for efficiently updating
/// the screen to reflect those changes.
#[derive(Debug)]
pub struct ScreenBuf {
    /// Grid of characters covering the screen.
    cells: Vec<Vec<DoubleCharCell>>,
    /// This should always contain the number of rows and cols requested by the
    /// user (eg. 0-by-5), even if `cells` is empty.
    size: Pos,
}

/// Represents a single character on a screen, with style properties.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CharCell {
    ch: char,
    shade: Shade,
    style: Style, // TODO except background is ignored?
}

/// Stores both the new unprinted state of a character, and the old state that was last printed to the screen.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct DoubleCharCell {
    new: CharCell,
    /// None if unknown (eg. because it's never been printed or the screen was just resized)
    old: Option<CharCell>,
}

/// Instructions for how to update a terminal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScreenOp {
    /// Print a character at the current cursor position, and advance the cursor by 1.
    Print(char),
    /// Set a persistent style that will apply to anything printed, until a new style is applied.
    Apply(Style),
    /// Set the cursor position.
    Goto(Pos),
}

/// States of the ScreenBufIter
enum State {
    FindDirty,
    CheckStyle,
    PrintChar,
}

/// An iterator that produces instructions for updating a screen to match changes in a ScreenBuf.
pub struct ScreenBufIter<'a> {
    buf: &'a mut ScreenBuf,
    /// The last style applied to the screen. It will persist until a new style is applied.
    current_style: Option<Style>,
    /// On the very first iteration, we don't know where the terminal cursor is
    /// yet. We have to set it after we find the first dirty character cell,
    /// before printing the character.
    on_first_iteration: bool,
    /// Which cell the iterator is considering (NOT the position of the
    /// terminal's cursor). None means the iterator has just been constructed
    /// and is pointing to immediately before the first cell of the ScreenBuf.
    pos: Option<Pos>,
    /// Which action the iterator should do next.
    next_state: State,
}

enum FindDirtyResult {
    AtEnd,
    GotoDirty(ScreenOp),
    AtDirty,
}

impl ScreenBuf {
    pub fn new() -> Self {
        ScreenBuf {
            cells: Vec::new(),
            size: Pos::zero(),
        }
    }

    /// Get `ScreenOp` instructions that describe all changes to the screen buffer since the last time this method was called.
    pub fn drain_changes(&mut self) -> ScreenBufIter {
        ScreenBufIter {
            buf: self,
            current_style: None,
            pos: None,
            on_first_iteration: true,
            next_state: State::FindDirty,
        }
    }

    pub fn resize(&mut self, size: Pos) {
        self.cells = Vec::new();
        let mut row = Vec::new();
        row.resize_with(size.col as usize, Default::default);
        self.cells.resize(size.row as usize, row);
        self.size = size;
    }

    pub fn size(&self) -> Pos {
        self.size
    }

    pub fn write_str(&mut self, pos: Pos, s: &str, style: Style) -> Result<(), Error> {
        let mut maybe_pos = Ok(pos);
        for ch in s.chars() {
            let p = maybe_pos?;
            self.set_char_with_style(p, ch, style)?;
            maybe_pos = self.next_pos(p).ok_or(Error::OutOfBounds);
        }
        Ok(())
    }

    pub fn shade_region(&mut self, region: Region, shade: Shade) -> Result<(), Error> {
        for pos in region.positions() {
            self.get_mut(pos)?.set_shade(shade);
        }
        Ok(())
    }

    pub fn style_region(&mut self, region: Region, style: Style) -> Result<(), Error> {
        for pos in region.positions() {
            self.get_mut(pos)?.set_style(style);
        }
        Ok(())
    }

    fn set_char_with_style(&mut self, pos: Pos, ch: char, style: Style) -> Result<(), Error> {
        let cell = self.get_mut(pos)?;
        cell.set_char(ch);
        cell.set_style(style);
        Ok(())
    }

    fn get(&self, pos: Pos) -> Result<DoubleCharCell, Error> {
        self.cells
            .get(pos.row as usize)
            .and_then(|row| row.get(pos.col as usize))
            .map(|cell| *cell)
            .ok_or(Error::OutOfBounds)
    }

    fn get_mut(&mut self, pos: Pos) -> Result<&mut DoubleCharCell, Error> {
        self.cells
            .get_mut(pos.row as usize)
            .and_then(|row| row.get_mut(pos.col as usize))
            .ok_or(Error::OutOfBounds)
    }

    fn next_pos(&self, old_pos: Pos) -> Option<Pos> {
        let size = self.size();
        if old_pos.col >= (size.col - 1) {
            // At the last column of a row
            if old_pos.row >= (size.row - 1) {
                // At the last row too, that's the last position on the the screen!
                None
            } else {
                // Go to start of next row
                Some(Pos {
                    row: old_pos.row + 1,
                    col: 0,
                })
            }
        } else {
            // Go forward 1 column
            Some(Pos {
                row: old_pos.row,
                col: old_pos.col + 1,
            })
        }
    }
}

impl CharCell {
    fn shaded_style(&self) -> Style {
        let mut style = self.style;
        style.shade = self.shade;
        style
    }
}

impl DoubleCharCell {
    fn set_char(&mut self, ch: char) {
        self.new.ch = ch;
    }

    fn set_style(&mut self, style: Style) {
        self.new.style = style;
    }

    fn set_shade(&mut self, shade: Shade) {
        self.new.shade = shade;
    }

    fn get(&self) -> CharCell {
        self.new
    }

    fn mark(&mut self) {
        self.old = Some(self.new);
        self.new = CharCell::default();
    }

    fn is_dirty(&self) -> bool {
        if let Some(old) = self.old {
            self.new != old
        } else {
            true
        }
    }
}

impl Default for CharCell {
    fn default() -> Self {
        CharCell {
            ch: ' ',
            shade: Shade::default(),
            style: Style::default(),
        }
    }
}

impl<'a> ScreenBufIter<'a> {
    fn advance(&mut self) -> Option<Pos> {
        if let Some(p) = self.pos {
            self.buf.get_mut(p).unwrap().mark();
            self.pos = Some(self.buf.next_pos(p)?);
        } else {
            // Start at the beginning
            self.pos = Some(Pos::zero())
        }
        self.pos
    }

    fn pos(&self) -> Pos {
        // If pos is None, the iterator is still _before_ the first cell, and not _at_ any cell.
        self.pos
            .expect("position not defined until advance() is called")
    }

    fn cell(&self) -> CharCell {
        self.buf.get(self.pos()).unwrap().get()
    }

    fn at_dirty_cell(&self) -> bool {
        self.buf.get(self.pos()).unwrap().is_dirty()
    }

    fn find_dirty(&mut self) -> FindDirtyResult {
        // Look for the next cell after this one that needs to be redisplayed.
        if self.advance().is_none() {
            return FindDirtyResult::AtEnd;
        }

        let mut jumped = false;
        while !self.at_dirty_cell() {
            if self.advance().is_none() {
                return FindDirtyResult::AtEnd;
            }
            jumped = true;
        }

        // Check if we need to explicitly jump the cursor to this position.
        if jumped || self.on_first_iteration {
            self.on_first_iteration = false;
            FindDirtyResult::GotoDirty(ScreenOp::Goto(self.pos()))
        } else {
            FindDirtyResult::AtDirty
        }
    }

    fn check_style(&mut self) -> Option<ScreenOp> {
        // Check if it has a different style than the last one we applied.
        let style = self.cell().shaded_style();
        let style_changed = match self.current_style {
            None => true,
            Some(s) => s != style,
        };

        if style_changed {
            self.current_style = Some(style);
            assert!(!self.on_first_iteration);
            Some(ScreenOp::Apply(style))
        } else {
            None
        }
    }

    fn print_char(&mut self) -> ScreenOp {
        // Finally, print the character itself
        assert!(!self.on_first_iteration);
        ScreenOp::Print(self.cell().ch)
    }
}

impl<'a> Iterator for ScreenBufIter<'a> {
    type Item = ScreenOp;
    fn next(&mut self) -> Option<ScreenOp> {
        loop {
            match self.next_state {
                State::FindDirty => {
                    self.next_state = State::CheckStyle;
                    match self.find_dirty() {
                        FindDirtyResult::GotoDirty(op) => {
                            return Some(op);
                        }
                        FindDirtyResult::AtDirty => (), // No Goto op needed, continue to next state
                        FindDirtyResult::AtEnd => {
                            // Done! Reached the end of the buffer.
                            return None;
                        }
                    }
                }
                State::CheckStyle => {
                    self.next_state = State::PrintChar;
                    let op = self.check_style();
                    if op.is_some() {
                        return op;
                    }
                }
                State::PrintChar => {
                    self.next_state = State::FindDirty;
                    return Some(self.print_char());
                }
            }
        }
    }
}

#[cfg(test)]
mod screen_buf_tests {
    use super::*;
    use pretty::{Bound, Color, Pos, Region, Shade, Style};

    fn assert_out_of_bounds(result: Result<(), Error>) {
        match result {
            Err(Error::OutOfBounds) => (),
            x => panic!("expected OutOfBounds error, got {:?}", x),
        }
    }

    fn assert_resized(buf: &mut ScreenBuf, size: Pos, good_pos: &[Pos], bad_pos: &[Pos]) {
        buf.resize(size);
        assert_eq!(buf.size(), size);
        for &pos in good_pos {
            buf.set_char_with_style(pos, 'x', Style::default())
                .expect(&format!(
                    "pos {} out-of-bounds of buf with size {}",
                    pos, size
                ));
        }
        for &pos in bad_pos {
            assert_out_of_bounds(buf.set_char_with_style(pos, 'x', Style::default()));
        }
    }

    #[test]
    fn test_resize() {
        let c0r1 = Pos { col: 0, row: 1 };
        let c1r0 = Pos { col: 1, row: 0 };
        let c1r1 = Pos { col: 1, row: 1 };
        let c5r8 = Pos { col: 5, row: 8 };
        let c5r7 = Pos { col: 5, row: 7 };
        let c4r7 = Pos { col: 4, row: 7 };
        let c4r8 = Pos { col: 4, row: 8 };

        let mut buf = ScreenBuf::new();
        assert_eq!(buf.size(), Pos::zero());
        assert_out_of_bounds(buf.set_char_with_style(Pos::zero(), 'x', Style::default()));

        assert_resized(&mut buf, Pos::zero(), &[], &[Pos::zero(), c1r0, c0r1]);
        assert_resized(&mut buf, c1r0, &[], &[Pos::zero(), c1r0, c0r1]);
        assert_resized(&mut buf, c0r1, &[], &[Pos::zero(), c1r0, c0r1]);
        assert_resized(&mut buf, c1r1, &[Pos::zero()], &[c1r0, c0r1, c1r1]);
        assert_resized(
            &mut buf,
            c5r8,
            &[Pos::zero(), c1r0, c0r1, c1r1, c4r7],
            &[c5r8, c4r8, c5r7],
        );
        assert_resized(&mut buf, c1r1, &[Pos::zero()], &[c1r0, c0r1, c1r1]);
    }

    #[test]
    fn test_simple() {
        let style1 = Style::color(Color::Base09);
        let mut buf = ScreenBuf::new();
        buf.resize(Pos { col: 3, row: 2 });

        let pos = Pos { col: 2, row: 0 };
        buf.write_str(pos, "x", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Apply(style1),
                ScreenOp::Print('x'),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
            ]
        );
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(pos),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
            ]
        );
    }

    #[test]
    fn test_no_change() {
        let style1 = Style::color(Color::Base09);
        let mut buf = ScreenBuf::new();
        buf.resize(Pos { col: 3, row: 2 });

        let pos = Pos { col: 2, row: 0 };
        buf.write_str(pos, "x", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();

        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Apply(style1),
                ScreenOp::Print('x'),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
            ]
        );

        // Print same thing as before
        buf.write_str(pos, "x", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(actual_ops, vec![]);
    }

    #[test]
    fn test_shorten() {
        let style1 = Style::color(Color::Base09);
        let mut buf = ScreenBuf::new();
        buf.resize(Pos { col: 3, row: 1 });

        buf.write_str(Pos::zero(), "xyz", style1).unwrap();
        let mut actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(style1),
                ScreenOp::Print('x'),
                ScreenOp::Print('y'),
                ScreenOp::Print('z'),
            ]
        );

        buf.write_str(Pos::zero(), "xy", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 2, row: 0 }),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
            ]
        );

        buf.write_str(Pos::zero(), "x", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
            ]
        );

        buf.write_str(Pos::zero(), "xy", style1).unwrap();
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Apply(style1),
                ScreenOp::Print('y'),
            ]
        );
    }

    #[test]
    fn test_complex() {
        let style1 = Style::color(Color::Base09);
        let style2 = Style::color(Color::Base0C);

        let mut buf = ScreenBuf::new();
        buf.resize(Pos { col: 3, row: 4 });

        buf.write_str(Pos { col: 1, row: 0 }, "foobar", style1)
            .unwrap();

        buf.write_str(Pos { col: 0, row: 1 }, "OB", style2).unwrap();

        buf.write_str(Pos { col: 2, row: 3 }, "$", Style::default())
            .unwrap();

        let mut actual_ops: Vec<_> = buf.drain_changes().collect();

        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Apply(style1),
                ScreenOp::Print('f'),
                ScreenOp::Print('o'),
                ScreenOp::Apply(style2),
                ScreenOp::Print('O'),
                ScreenOp::Print('B'),
                ScreenOp::Apply(style1),
                ScreenOp::Print('a'),
                ScreenOp::Print('r'),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print('$'),
            ]
        );
        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 0 }),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { col: 2, row: 3 }),
                ScreenOp::Print(' '),
            ]
        );

        buf.style_region(Region::char_region(Pos { col: 0, row: 2 }), style2)
            .unwrap();
        buf.set_char_with_style(Pos { col: 2, row: 3 }, '!', Style::default())
            .unwrap();

        actual_ops = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 0, row: 2 }),
                ScreenOp::Apply(style2),
                ScreenOp::Print(' '),
                ScreenOp::Goto(Pos { col: 2, row: 3 }),
                ScreenOp::Apply(Style::default()),
                ScreenOp::Print('!'),
            ]
        );
    }

    #[test]
    fn test_shade_region() {
        let style1 = Style::color(Color::Base09);
        let style2 = Style::color(Color::Base0C);
        let cursor = Shade(0);

        let mut buf = ScreenBuf::new();
        buf.resize(Pos { col: 4, row: 3 });

        // Write something with some style and the default background shade.
        buf.write_str(Pos::zero(), "0123456789ab", style1).unwrap();

        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos::zero()),
                ScreenOp::Apply(style1),
                ScreenOp::Print('0'),
                ScreenOp::Print('1'),
                ScreenOp::Print('2'),
                ScreenOp::Print('3'),
                ScreenOp::Print('4'),
                ScreenOp::Print('5'),
                ScreenOp::Print('6'),
                ScreenOp::Print('7'),
                ScreenOp::Print('8'),
                ScreenOp::Print('9'),
                ScreenOp::Print('a'),
                ScreenOp::Print('b'),
            ]
        );

        // Rewrite it, but change background to cursor-shade in some region
        let region = Region {
            pos: Pos { col: 1, row: 1 },
            bound: Bound {
                width: 3,
                height: 2,
                indent: 1,
            },
        };
        buf.write_str(Pos::zero(), "0123456789ab", style1).unwrap();
        buf.shade_region(region, cursor).unwrap();

        // Ensure that the shade overrides the original style within the cursor region
        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 1, row: 1 }),
                ScreenOp::Apply(Style {
                    shade: cursor,
                    ..style1
                }),
                ScreenOp::Print('5'),
                ScreenOp::Print('6'),
                ScreenOp::Print('7'),
                ScreenOp::Goto(Pos { col: 1, row: 2 }),
                ScreenOp::Print('9'),
            ]
        );

        // Add new text with a different style, overlapping the cursor region
        buf.write_str(Pos::zero(), "0123456789ab", style1).unwrap();
        buf.shade_region(region, cursor).unwrap();
        buf.write_str(Pos { col: 0, row: 1 }, "xyz", style2)
            .unwrap();

        // Ensure that the shade overrides the new style within the cursor region
        let actual_ops: Vec<_> = buf.drain_changes().collect();
        assert_eq!(
            actual_ops,
            vec![
                ScreenOp::Goto(Pos { col: 0, row: 1 }),
                ScreenOp::Apply(style2),
                ScreenOp::Print('x'),
                ScreenOp::Apply(Style {
                    shade: cursor,
                    ..style2
                }),
                ScreenOp::Print('y'),
                ScreenOp::Print('z'),
            ]
        );
    }
}
