use std::iter;

/// This enum provides a consistent interface to a piece of text, while allowing
/// various optimizations that depend on whether the text is actively being
/// edited or not.
#[derive(Clone, Debug)]
pub enum Text {
    /// Text that is not currently being edited.
    Inactive(String),
    /// Text that is currently being edited.
    Active(ActiveText),
}

/// Text that is currently being edited.
#[derive(Clone, Debug)]
pub struct ActiveText(String);

impl Text {
    /// Construct a new `Text` in an inactive state.
    pub fn new_inactive() -> Self {
        Text::Inactive(String::new())
    }

    /// Switch from an inactive to active state, so the text can be edited.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is already active.
    pub fn activate(&mut self) {
        *self = match self {
            Text::Inactive(s) => Text::Active(ActiveText(s.to_owned())),
            _ => panic!("text is already active"),
        }
    }

    /// Switch from an active to inactive state, so the text can no longer be edited.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is already inactive.
    pub fn deactivate(&mut self) {
        *self = match self {
            Text::Active(ActiveText(s)) => Text::Inactive(s.to_owned()),
            _ => panic!("text is already inactive"),
        }
    }

    /// Get a reference to the underlying string. Works both when active and inactive.
    pub fn as_str(&self) -> &str {
        match self {
            Text::Active(ref s) => &s.0,
            Text::Inactive(ref s) => s,
        }
    }

    /// Return the length of the text in characters. Works both when active and inactive.
    pub fn num_chars(&self) -> usize {
        match self {
            Text::Active(ref s) => s.num_chars(),
            Text::Inactive(ref s) => s.chars().count(),
        }
    }

    /// Insert a new character at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is inactive or the index is greater than the number
    /// of characters in the text.
    pub fn insert(&mut self, char_index: usize, character: char) {
        match self {
            Text::Active(active_text) => active_text.insert(char_index, character),
            _ => panic!("Text::insert - tried to edit inactive text"),
        }
    }

    /// Remove and return the character at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is inactive or the index is greater than the number
    /// of characters in the text.
    pub fn delete(&mut self, char_index: usize) -> char {
        match self {
            Text::Active(active_text) => active_text.delete(char_index),
            _ => panic!("Text::delete - tried to edit inactive text"),
        }
    }

    /// Set the text to the given string, replacing the current contents.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is inactive.
    pub fn set(&mut self, s: String) {
        match self {
            Text::Active(active_text) => active_text.set(s),
            _ => panic!("Text::set - tried to edit inactive text"),
        }
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        match self {
            Text::Active(s) => &s.0,
            Text::Inactive(s) => s,
        }
    }
}

impl ActiveText {
    fn num_chars(&self) -> usize {
        self.0.chars().count()
    }

    fn delete(&mut self, char_index: usize) -> char {
        // If char_index is larger than that, let byte_index() panic!
        self.0.remove(self.byte_index(char_index))
    }

    fn insert(&mut self, char_index: usize, character: char) {
        self.0.insert(self.byte_index(char_index), character);
    }

    fn set(&mut self, s: String) {
        self.0 = s;
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.0
            .char_indices()
            .map(|(i, _)| i)
            .chain(iter::once(self.0.len()))
            .nth(char_index)
            .expect("ActiveText::byte_index - character index is out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Text::insert - tried to edit inactive text")]
    fn test_inactive_insert() {
        let mut t = Text::new_inactive();
        t.insert(0, 'a');
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_insert() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(1, 'c');
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_insert2() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.insert(2, 'c');
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_insert3() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.insert(3, 'd');
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete() {
        let mut t = Text::new_inactive();
        t.activate();
        t.delete(1);
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete2() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.delete(2);
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete3() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.delete(3);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_active_edit() {
        let mut t = Text::new_inactive();
        t.activate();
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

        t.deactivate();
        assert_eq!(t.num_chars(), 5);
        assert_eq!(t.as_str(), "abcde");
        t.activate();
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
