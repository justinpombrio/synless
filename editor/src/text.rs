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
    pub fn inactivate(&mut self) {
        *self = match self {
            Text::Active(ActiveText(s)) => Text::Inactive(s.to_owned()),
            _ => panic!("text is already inactive"),
        }
    }

    /// Get a reference to the underlying string. Works both when active and inactive.
    // TODO this might not be possible if we decide to store active text in a fancier data structure...
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

    /// Remove and return the character at the given index, or `None` if the index
    /// is equal to the number of characters in the text.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is inactive or the index is greater than the number
    /// of characters in the text.
    pub fn delete_forward(&mut self, char_index: usize) -> Option<char> {
        match self {
            Text::Active(active_text) => active_text.delete_forward(char_index),
            _ => panic!("Text::delete_forward - tried to edit inactive text"),
        }
    }

    /// Remove and return the character immediately before the given index, or
    /// `None` if the index is 0.
    ///
    /// # Panics
    ///
    /// Panics if the `Text` is inactive or the index is greater than the number
    /// of characters in the text.
    pub fn delete_backward(&mut self, char_index: usize) -> Option<char> {
        if let Text::Active(active_text) = self {
            active_text.delete_backward(char_index)
        } else {
            panic!("Text::delete_backward - tried to edit inactive text")
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

    fn delete_forward(&mut self, char_index: usize) -> Option<char> {
        if char_index == self.num_chars() {
            return None;
        }
        // If char_index is larger than that, let byte_index() panic!
        Some(self.0.remove(self.byte_index(char_index)))
    }

    fn delete_backward(&mut self, char_index: usize) -> Option<char> {
        if char_index == 0 {
            return None;
        }
        // This exact index makes `String` panic rather than `byte_index`.
        if char_index == self.num_chars() + 1 {
            panic!("ActiveText::delete_backward - character index is out of range")
        }
        // If char_index is larger than that, let byte_index() panic!
        Some(self.0.remove(self.byte_index(char_index - 1)))
    }

    fn insert(&mut self, char_index: usize, character: char) {
        self.0.insert(self.byte_index(char_index), character);
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
    #[should_panic(expected = "ActiveText::delete_backward - character index is out of range")]
    fn test_active_invalid_delete_backward() {
        let mut t = Text::new_inactive();
        t.activate();
        t.delete_backward(1);
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete_backward2() {
        let mut t = Text::new_inactive();
        t.activate();
        t.delete_backward(2);
    }

    #[test]
    #[should_panic(expected = "ActiveText::delete_backward - character index is out of range")]
    fn test_active_invalid_delete_backward3() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.delete_backward(2);
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete_forward() {
        let mut t = Text::new_inactive();
        t.activate();
        t.delete_forward(1);
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete_forward2() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.delete_forward(2);
    }

    #[test]
    #[should_panic(expected = "ActiveText::byte_index - character index is out of range")]
    fn test_active_invalid_delete_forward3() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.delete_forward(3);
    }

    #[test]
    #[should_panic(expected = "Text::delete_forward - tried to edit inactive text")]
    fn test_inactivate() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'a');
        t.inactivate();
        t.delete_forward(0);
    }

    #[test]
    fn test_active_edit() {
        let mut t = Text::new_inactive();
        t.activate();
        t.insert(0, 'b');
        t.insert(0, 'a');
        t.insert(2, 'd');
        assert_eq!("abd", t.as_str());

        t.insert(2, 'c');
        assert_eq!("abcd", t.as_str());

        t.insert(4, 'e');
        assert_eq!("abcde", t.as_str());

        t.inactivate();
        assert_eq!(5, t.num_chars());
        t.activate();
        assert_eq!(5, t.num_chars());

        assert_eq!(Some('c'), t.delete_forward(2));
        assert_eq!("abde", t.as_str());

        assert_eq!(Some('b'), t.delete_backward(2));
        assert_eq!("ade", t.as_str());

        assert_eq!(Some('a'), t.delete_forward(0));
        assert_eq!("de", t.as_str());

        assert_eq!(None, t.delete_backward(0));
        assert_eq!("de", t.as_str());
        assert_eq!(None, t.delete_forward(2));
        assert_eq!("de", t.as_str());

        assert_eq!(Some('e'), t.delete_backward(2));
        assert_eq!("d", t.as_str());

        assert_eq!(Some('d'), t.delete_backward(1));
        assert_eq!("", t.as_str());

        assert_eq!(None, t.delete_backward(0));
        assert_eq!("", t.as_str());

        t.insert(0, 'a');
        assert_eq!(Some('a'), t.delete_forward(0));
        assert_eq!("", t.as_str());

        assert_eq!(None, t.delete_forward(0));
        assert_eq!("", t.as_str());

        assert_eq!(0, t.num_chars());
    }
}
