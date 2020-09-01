use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;
use thiserror::Error;

lazy_static! {
    static ref DATA_ITEM_REGEX: Regex =
        Regex::new(r##"^((?P<binary>[01]{5,6})|((0\.(?P<real_first>\d{6}))( (0\.(?P<real_second>\d{6})))( (0\.(?P<real_third>\d{6})))( (0\.(?P<real_fourth>\d{6})))( (0\.(?P<real_fifth>\d{6})))( (0\.(?P<real_sixth>\d{6})))))$"##)
            .unwrap();
}

/// A `DataItem` is just an ascii string
/// While working with this struct, it is assumed that all the characters in the string are:
///     1. Valid uf8(Rust takes care of this since all the Strings in rust are valid ut8)
///     2. Ascii digits or dot
#[derive(Debug, PartialEq, Clone)]
pub enum DataItem {
    Binary(String),
    Real(String),
}

impl DataItem {
    pub const IGNORED_CHAR: char = '.';

    pub fn as_str(&self) -> &str {
        match self {
            Self::Binary(binary) => binary,
            Self::Real(real) => real,
        }
    }

    /// Gets a character at an index. Returns none if it is out of range
    fn char_at(&self, index: usize) -> Option<char> {
        self.as_str().chars().nth(index)
    }

    fn is_binary(&self) -> bool {
        match self {
            Self::Binary(_) => true,
            _ => false,
        }
    }

    fn is_real(&self) -> bool {
        match self {
            Self::Real(_) => true,
            _ => false,
        }
    }

    fn width(&self) -> usize {
        self.as_str().len()
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum DataItemParseError {
    #[error("not valid ascii")]
    NotValidAscii,

    #[error("invalid format")]
    InvalidFormat,
}

// Allow an string types to be converted(falliably) to a DataItem
impl FromStr for DataItem {
    type Err = DataItemParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if !input.is_ascii() {
            return Err(DataItemParseError::NotValidAscii);
        }

        match DATA_ITEM_REGEX.captures(input) {
            Some(captures) => {
                if let Some(binary) = captures.name("binary") {
                    Ok(DataItem::Binary(binary.as_str().to_owned()))
                } else if let (
                    Some(first),
                    Some(second),
                    Some(third),
                    Some(fourth),
                    Some(fifth),
                    Some(sixth),
                ) = (
                    captures.name("real_first"),
                    captures.name("real_second"),
                    captures.name("real_third"),
                    captures.name("real_fourth"),
                    captures.name("real_fifth"),
                    captures.name("real_sixth"),
                ) {
                    let first = first.as_str();
                    let second = second.as_str();
                    let third = third.as_str();
                    let fourth = fourth.as_str();
                    let fifth = fifth.as_str();
                    let sixth = sixth.as_str();

                    let mut input = String::with_capacity(
                        first.len()
                            + second.len()
                            + third.len()
                            + fourth.len()
                            + fifth.len()
                            + sixth.len(),
                    );

                    input.push_str(first);
                    input.push_str(second);
                    input.push_str(third);
                    input.push_str(fourth);
                    input.push_str(fifth);
                    input.push_str(sixth);

                    Ok(DataItem::Real(input))
                } else {
                    unreachable!("shouldn't be reached")
                }
            }
            None => Err(DataItemParseError::InvalidFormat),
        }
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
        assert_eq!(
            "abc".parse::<DataItem>(),
            Err(DataItemParseError::InvalidFormat)
        );
    }

    #[test]
    fn test_one_ignored() {
        assert_eq!(
            "0.981136 0.369132 0.498354 0.067417 0.422276 0.803662".parse::<DataItem>(),
            Ok(DataItem::Real(
                "981136369132498354067417422276803662".to_owned()
            ))
        );
    }

    #[test]
    fn test_is_binary() {
        let data_item = DataItem::from_str("0.981136 0.369132 0.498354 0.067417 0.422276 0.803662")
            .expect("data item input is invalid");
        assert_eq!(data_item.is_binary(), false);

        let data_item = DataItem::from_str("00010").expect("data item input is invalid");
        assert_eq!(data_item.is_binary(), true);
    }

    #[test]
    fn test_num_digits_width() {
        let data_item = DataItem::from_str("00001").expect("data item input is invalid");
        assert_eq!(data_item.width(), 5);

        let data_item = DataItem::from_str("0.981136 0.369132 0.498354 0.067417 0.422276 0.803662")
            .expect("data item input is invalid");
        assert_eq!(data_item.width(), 36);
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
        let data_item = DataItem::from_str("0.981136 0.369132 0.498354 0.067417 0.422276 0.803662")
            .expect("data item input is invalid");
        assert_eq!(data_item.char_at(0), Some('9'));
        assert_eq!(data_item.char_at(37), None);
    }
}
