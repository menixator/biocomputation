use std::str::FromStr;
use thiserror::Error;

/// A `DataItem` is just an ascii string
/// While working with this struct, it is assumed that all the characters in the string are:
///     1. Valid uf8(Rust takes care of this since all the Strings in rust are valid ut8)
///     2. Ascii digits or dot
#[derive(Debug, PartialEq)]
pub enum DataItem {
    Binary(String),
    RealOrInt(String),
}

impl DataItem {
    pub const IGNORED_CHAR: char = '.';

    pub fn as_str(&self) -> &str {
        match self {
            Self::Binary(binary) => binary,
            Self::RealOrInt(real) => real,
        }
    }

    /// Gets a character at an index. Returns none if it is out of range
    fn char_at(&self, index: usize) -> Option<char> {
        self.as_str().chars().nth(index)
    }

    /// Checks if a particular index is ignored. Returns false is the index is out of range.
    fn is_index_ignored(&self, index: usize) -> bool {
        match self.char_at(index) {
            None => false,
            Some(character) => character == Self::IGNORED_CHAR,
        }
    }

    fn is_binary(&self) -> bool {
        match self {
            Self::Binary(_) => true,
            _ => false,
        }
    }

    fn is_real_or_int(&self) -> bool {
        match self {
            Self::RealOrInt(_) => true,
            _ => false,
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

        let mut is_binary = true;
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
            } else {
                if is_binary {
                    is_binary = character == '0' || character == '1';
                }
            }
        }

        let input = input.to_owned();
        Ok(if is_binary && !found_ignored_char {
            DataItem::Binary(input)
        } else {
            DataItem::RealOrInt(input)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_binary() {
        assert_eq!("00000".parse(), Ok(DataItem::Binary("00000".to_owned())));
    }

    #[test]
    fn test_non_digit() {
        assert_eq!("abc".parse::<DataItem>(), Err(DataItemParseError::NotDigit));
    }

    #[test]
    fn test_one_ignored() {
        assert_eq!(
            "000.".parse::<DataItem>(),
            Ok(DataItem::RealOrInt("000.".to_owned()))
        );
    }

    #[test]
    fn test_is_binary() {
        let data_item = DataItem::from_str("0123.0").expect("data item input is invalid");
        assert_eq!(data_item.is_binary(), false);

        let data_item = DataItem::from_str("01230").expect("data item input is invalid");
        assert_eq!(data_item.is_binary(), false);

        let data_item = DataItem::from_str("00010.1").expect("data item input is invalid");
        assert_eq!(data_item.is_binary(), false);

        let data_item = DataItem::from_str("00010").expect("data item input is invalid");
        assert_eq!(data_item.is_binary(), true);
    }

    #[test]
    fn test_is_index_ignored() {
        let data_item = DataItem::from_str("0123.0").expect("data item input is invalid");
        assert_eq!(data_item.is_index_ignored(0), false);
        // Index in range
        assert_eq!(data_item.is_index_ignored(4), true);
        // Index out of range
        assert_eq!(data_item.is_index_ignored(6), false);
    }

    #[test]
    fn test_multiple_ignored() {
        assert_eq!(
            "000.1.1".parse::<DataItem>(),
            Err(DataItemParseError::MultipleIngoredChar)
        );
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
