use crate::infra::SynlessBug;
use std::iter;

#[derive(Clone, Debug)]
pub struct Text(String);

impl Text {
    pub fn new() -> Self {
        Text(String::new())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_split_str(&self, char_index: usize) -> (&str, &str) {
        self.0.as_str().split_at(self.byte_index(char_index))
    }

    /// Return the length of the text in characters.
    pub fn num_chars(&self) -> usize {
        self.0.chars().count()
    }

    /// Insert a new character at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is too large.
    pub fn insert(&mut self, char_index: usize, character: char) {
        self.0.insert(self.byte_index(char_index), character);
    }

    /// Remove and return the character at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is too large.
    pub fn delete(&mut self, char_index: usize) -> char {
        self.0.remove(self.byte_index(char_index))
    }

    /// Set the text to the given string, replacing the current contents.
    pub fn set(&mut self, s: String) {
        self.0 = s;
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.0
            .char_indices()
            .map(|(i, _)| i)
            // The byte index at the end of the string
            .chain(iter::once(self.0.len()))
            .nth(char_index)
            .bug_msg("Text - character index is out of range")
    }
}

impl Default for Text {
    fn default() -> Self {
        Text::new()
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
        assert_eq!(t.num_chars(), 3);
        assert_eq!(t.as_str(), "abd");
        assert_eq!(t.num_chars(), 3);

        t.insert(2, 'c');
        assert_eq!(t.as_str(), "abcd");

        t.insert(4, 'e');
        assert_eq!(t.as_str(), "abcde");

        assert_eq!(t.num_chars(), 5);
        assert_eq!(t.as_str(), "abcde");

        assert_eq!(t.delete(2), 'c');
        assert_eq!(t.as_str(), "abde");

        assert_eq!(t.delete(1), 'b');
        assert_eq!(t.as_str(), "ade");

        assert_eq!(t.delete(0), 'a');
        assert_eq!(t.num_chars(), 2);
        assert_eq!(t.as_str(), "de");

        assert_eq!(t.delete(1), 'e');
        assert_eq!(t.as_str(), "d");
        assert_eq!(t.num_chars(), 1);

        assert_eq!(t.delete(0), 'd');
        assert_eq!(t.num_chars(), 0);
        assert_eq!(t.as_str(), "");

        t.insert(0, 'a');
        assert_eq!(t.delete(0), 'a');
        assert_eq!(t.as_str(), "");
        assert_eq!(t.num_chars(), 0);
    }
}
