use std::str::FromStr;
use thiserror::Error;

/// A `DataItem` is just an ascii string
/// While working with this struct, it is assumed that all the characters in the string are:
///     1. Valid uf8(Rust takes care of this since all the Strings in rust are valid ut8)
///     2. Ascii digits or dot
#[derive(Debug, PartialEq)]
pub struct DataItem(String);

impl DataItem {
    pub const IGNORED_CHAR: char = '.';

    /// Gets a character at an index. Returns none if it is out of range
    fn char_at(&self, index: usize) -> Option<char> {
        self.0.chars().nth(index)
    }

    /// Checks if a particular index is ignored. Returns false is the index is out of range.
    fn is_index_ignored(&self, index: usize) -> bool {
        match self.char_at(index) {
            None => false,
            Some(character) => character == Self::IGNORED_CHAR,
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum DataItemParseError {
    #[error("not valid ascii")]
    NotValidAscii,

    #[error("not a digit")]
    NotDigit,

    #[error("multiple ignored characters present")]
    MultipleIngoredChar,
}

// Allow an string types to be converted(falliably) to a DataItem
impl FromStr for DataItem {
    type Err = DataItemParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if !input.is_ascii() {
            return Err(DataItemParseError::NotValidAscii);
        }

        let mut found_ignored_char = false;
        // if all characters are digits and only one ignored character is present, allow it
        for character in input.chars() {
            if character == Self::IGNORED_CHAR {
                if found_ignored_char {
                    return Err(DataItemParseError::MultipleIngoredChar);
                }
                found_ignored_char = true;
            } else if !character.is_ascii_digit() {
                return Err(DataItemParseError::NotDigit);
            }
        }

        Ok(DataItem(input.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ascii() {
        assert_eq!("00000".parse(), Ok(DataItem("00000".to_owned())));
    }

    #[test]
    fn test_non_ascii() {
        assert_eq!(
            "not_ascii❤".parse::<DataItem>(),
            Err(DataItemParseError::NotValidAscii)
        );
    }

    #[test]
    fn test_char_at() {
        let data_item = DataItem::from_str("12345").expect("data item input is invalid");
        assert_eq!(data_item.char_at(0), Some('1'));
        assert_eq!(data_item.char_at(5), None);
    }
}

/// A dataset is a container contains a list of `DataItem`s
pub struct DataSet(Vec<DataItem>);
