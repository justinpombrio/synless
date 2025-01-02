use regex::bytes;

use crate::language::{ReplacementTable, Storage};
use crate::util::{error, SynlessBug, SynlessError};
use std::iter;

#[derive(Clone, Debug)]
pub struct Text {
    source: IndexedString,
    display: Option<IndexedString>,
    table: Option<ReplacementTable>,
}

#[derive(Clone, Debug, Default)]
struct IndexedString {
    string: String,
    byte_index: usize,
}

#[derive(Clone, Debug)]
pub enum TextDeletion {
    Char(char),
    Seq(String),
}

#[derive(thiserror::Error, Debug)]
pub enum TextError {
    #[error("Character {0} is banned by the replacement table")]
    BannedCharacter(char),
    #[error("Sequence {0} is not in the replacement table")]
    NotInReplacementTable(String),
}

impl From<TextError> for SynlessError {
    fn from(error: TextError) -> SynlessError {
        error!(Edit, "{}", error)
    }
}

impl Text {
    pub fn new(table: Option<ReplacementTable>) -> Self {
        Text {
            source: IndexedString::default(),
            display: None,
            table,
        }
    }

    pub fn as_source_str(&self) -> &str {
        self.source.string.as_str()
    }

    pub fn as_display_str(&self) -> &str {
        self.display
            .as_ref()
            .map(|display| display.string.as_str())
            .unwrap_or(self.source.string.as_str())
    }

    /// Insert a new character at the cursor.
    pub fn insert_char(&mut self, s: &Storage, character: char) -> Result<(), TextError> {
        if let Some(table) = self.table {
            if table.is_banned(s, character) {
                return Err(TextError::BannedCharacter(character));
            }
        }

        self.source.insert_char(character);
        if let Some(display) = &mut self.display {
            display.insert_char(character);
        }
        Ok(())
    }

    /// Insert the given source string at the cursor. It must be present in the replacement table.
    pub fn insert_replacement_sequence(
        &mut self,
        s: &Storage,
        source_seq: &str,
    ) -> Result<(), TextError> {
        let display_seq = self
            .table
            .and_then(|table| table.source_to_display(s, source_seq))
            .ok_or_else(|| TextError::NotInReplacementTable(source_seq.to_owned()))?;

        if self.display.is_none() {
            self.display = Some(self.source.clone());
        }
        self.source.insert_str(source_seq);
        self.display.as_mut().bug().insert_str(&display_seq);
        Ok(())
    }

    #[must_use]
    pub fn backspace(&mut self, s: &Storage) -> Option<TextDeletion> {
        if let Some((source_len, display_len, is_seq)) = self.match_left(s) {
            self.move_left_bytes(source_len, display_len);
            Some(self.delete_bytes(source_len, display_len, is_seq))
        } else {
            None
        }
    }

    /// Remove and return the character before the cursor.
    #[must_use]
    pub fn delete(&mut self, s: &Storage) -> Option<TextDeletion> {
        if let Some((source_len, display_len, is_seq)) = self.match_right(s) {
            Some(self.delete_bytes(source_len, display_len, is_seq))
        } else {
            None
        }
    }

    #[must_use]
    pub fn move_left(&mut self, s: &Storage) -> bool {
        if let Some((source_len, display_len, _)) = self.match_left(s) {
            self.move_left_bytes(source_len, display_len);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn move_right(&mut self, s: &Storage) -> bool {
        if let Some((source_len, display_len, _)) = self.match_right(s) {
            self.move_right_bytes(source_len, display_len);
            true
        } else {
            false
        }
    }

    fn move_left_bytes(&mut self, source_len: usize, display_len: usize) {
        self.source.byte_index -= source_len;
        self.display
            .as_mut()
            .map(|display| display.byte_index -= display_len);
    }

    fn move_right_bytes(&mut self, source_len: usize, display_len: usize) {
        self.source.byte_index += source_len;
        self.display
            .as_mut()
            .map(|display| display.byte_index += display_len);
    }

    fn delete_bytes(
        &mut self,
        source_len: usize,
        display_len: usize,
        is_seq: bool,
    ) -> TextDeletion {
        if is_seq {
            let seq = self.source.delete_str(source_len);
            self.display
                .as_mut()
                .map(|display| display.delete_str(display_len));
            TextDeletion::Seq(seq)
        } else {
            let ch = self.source.string.remove(self.source.byte_index);
            self.display
                .as_mut()
                .map(|display| display.string.remove(display.byte_index));
            TextDeletion::Char(ch)
        }
    }

    // true means seq, false means char
    #[must_use]
    fn match_left(&mut self, s: &Storage) -> Option<(usize, usize, bool)> {
        let source_before_cursor = &self.source.string[0..self.source.byte_index];
        if self.display.is_some() && self.table.is_some() {
            if let Some((source_len, display_len)) =
                self.table.bug().match_at_end(s, source_before_cursor)
            {
                Some((source_len, display_len, true))
            } else if let Some(last_char) = source_before_cursor.chars().next_back() {
                Some((last_char.len_utf8(), last_char.len_utf8(), false))
            } else {
                None
            }
        } else {
            if let Some(last_char) = source_before_cursor.chars().next_back() {
                Some((last_char.len_utf8(), last_char.len_utf8(), false))
            } else {
                None
            }
        }
    }

    // true means seq, false means char
    #[must_use]
    fn match_right(&mut self, s: &Storage) -> Option<(usize, usize, bool)> {
        let source_after_cursor = &self.source.string[self.source.byte_index..];
        if self.display.is_some() && self.table.is_some() {
            if let Some((source_len, display_len)) =
                self.table.bug().match_at_start(s, source_after_cursor)
            {
                Some((source_len, display_len, true))
            } else if let Some(first_char) = source_after_cursor.chars().next() {
                Some((first_char.len_utf8(), first_char.len_utf8(), false))
            } else {
                None
            }
        } else {
            if let Some(first_char) = source_after_cursor.chars().next() {
                Some((first_char.len_utf8(), first_char.len_utf8(), false))
            } else {
                None
            }
        }
    }

    /// Set the text to the given string, replacing the current contents. It must not contain replacement sequences!
    pub fn set(&mut self, string: String) {
        self.source = IndexedString {
            string,
            byte_index: 0,
        };
        self.display = None;
    }

    /// Return the length of the source string in characters.
    #[cfg(test)]
    pub fn num_source_chars(&self) -> usize {
        self.source.string.chars().count()
    }
}

impl IndexedString {
    fn insert_char(&mut self, character: char) {
        self.string.insert(self.byte_index, character);
        self.byte_index += character.len_utf8();
    }

    fn insert_str(&mut self, string: &str) {
        self.string.insert_str(self.byte_index, string);
        self.byte_index += string.len();
    }

    fn delete_str(&mut self, bytes: usize) -> String {
        let range = self.byte_index..(self.byte_index + bytes);
        let seq = self.string[range.clone()].to_owned();
        self.string.replace_range(range, "");
        seq
    }
}

impl Default for Text {
    fn default() -> Self {
        Text::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Text - character index is out of range")]
    fn test_invalid_insert() {
        let mut t = Text::new();
        t.insert(1, 'c');
    }

    #[test]
    #[should_panic(expected = "Text - character index is out of range")]
    fn test_invalid_insert_2() {
        let mut t = Text::new();
        t.insert(0, 'a');
        t.insert(2, 'c');
    }

    #[test]
    #[should_panic(expected = "Text - character index is out of range")]
    fn test_invalid_delete() {
        let mut t = Text::new();
        t.delete(1);
    }

    #[test]
    #[should_panic(expected = "Text - character index is out of range")]
    fn test_invalid_delete_2() {
        let mut t = Text::new();
        t.insert(0, 'a');
        t.delete(2);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_edit() {
        let mut t = Text::new();
        t.insert(0, 'b');
        t.insert(0, 'a');
        t.insert(2, 'd');
        assert_eq!(t.as_str(), "abd");
        assert_eq!(t.num_source_chars(), 3);
        assert_eq!(t.as_str(), "abd");
        assert_eq!(t.num_source_chars(), 3);

        t.insert(2, 'c');
        assert_eq!(t.as_str(), "abcd");

        t.insert(4, 'e');
        assert_eq!(t.as_str(), "abcde");

        assert_eq!(t.num_source_chars(), 5);
        assert_eq!(t.as_str(), "abcde");

        assert_eq!(t.delete(2), 'c');
        assert_eq!(t.as_str(), "abde");

        assert_eq!(t.delete(1), 'b');
        assert_eq!(t.as_str(), "ade");

        assert_eq!(t.delete(0), 'a');
        assert_eq!(t.num_source_chars(), 2);
        assert_eq!(t.as_str(), "de");

        assert_eq!(t.delete(1), 'e');
        assert_eq!(t.as_str(), "d");
        assert_eq!(t.num_source_chars(), 1);

        assert_eq!(t.delete(0), 'd');
        assert_eq!(t.num_source_chars(), 0);
        assert_eq!(t.as_str(), "");

        t.insert(0, 'a');
        assert_eq!(t.delete(0), 'a');
        assert_eq!(t.as_str(), "");
        assert_eq!(t.num_source_chars(), 0);
    }
}
