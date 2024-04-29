use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        // validate the input

        // 1. check not empty

        let is_empty = s.trim().is_empty();

        // 2. check length

        let is_too_long = s.graphemes(true).count() > 256;

        // 3. check for invalid chars

        let invalid_chars = [
            '/', '{', '}', '[', ']', '(', ')', '.', ',', '|', '\\', '"', '<', '>',
        ];

        let has_invalid_char = s.chars().any(|grapheme| invalid_chars.contains(&grapheme));

        if is_empty || is_too_long || has_invalid_char {
            Err(format!("{} is not a valid subcriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
    pub fn inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ë".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "ë".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_sucessfully() {
        let name = "Abhishek Roy";
        assert_ok!(SubscriberName::parse(name.to_string()));
    }
}
