use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;
use thiserror::Error;

lazy_static! {
    static ref DATA_ITEM_REGEX: Regex =
        Regex::new(r##"^((?P<binary>[01]{5,6})|((?P<real_first>0\.\d{6})( (?P<real_second>0\.\d{6}))( (?P<real_third>0\.\d{6}))( (?P<real_fourth>0\.\d{6}))( (?P<real_fifth>0\.\d{6}))( (?P<real_sixth>0\.\d{6})))) (?P<output>[01])$"##)
            .unwrap();
}

/// A `DataItem` is just an ascii string
/// While working with this struct, it is assumed that all the characters in the string are:
///     1. Valid uf8(Rust takes care of this since all the Strings in rust are valid ut8)
///     2. Ascii digits or dot
#[derive(Debug, PartialEq, Clone)]
pub struct DataItem {
    input: String,
    output: String,
}

impl DataItem {
    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn as_str(&self) -> &str {
        &self.input
    }

    /// Gets a character at an index. Returns none if it is out of range
    pub fn char_at(&self, index: usize) -> Option<char> {
        self.as_str().chars().nth(index)
    }

    pub fn width(&self) -> usize {
        self.as_str().len()
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum DataItemParseError {
    #[error("not valid ascii")]
    NotValidAscii,

    #[error("invalid format")]
    InvalidFormat,

    #[error("float parse error")]
    FloatParseError(#[from] std::num::ParseFloatError),
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
                if let (Some(binary), Some(output)) =
                    (captures.name("binary"), captures.name("output"))
                {
                    Ok(DataItem {
                        input: binary.as_str().to_owned(),
                        output: output.as_str().to_owned(),
                    })
                } else if let (
                    Some(first),
                    Some(second),
                    Some(third),
                    Some(fourth),
                    Some(fifth),
                    Some(sixth),
                    Some(output),
                ) = (
                    captures.name("real_first"),
                    captures.name("real_second"),
                    captures.name("real_third"),
                    captures.name("real_fourth"),
                    captures.name("real_fifth"),
                    captures.name("real_sixth"),
                    captures.name("output"),
                ) {
                    let all = [first, second, third, fourth, fifth, sixth];
                    let results: Result<Vec<f64>, std::num::ParseFloatError> = all
                        .iter()
                        .map(|capture| capture.as_str())
                        .map(|float_as_str| float_as_str.parse::<f64>())
                        .collect();
                    let results = results?;
                    let input: String = results
                        .into_iter()
                        .map(|float_value| float_value.round())
                        .map(|binary_float| binary_float as usize)
                        .map(|binary_usize| if binary_usize == 0 { "0" } else { "1" })
                        .collect();

                    Ok(DataItem {
                        input,
                        output: output.as_str().to_owned(),
                    })
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
        assert_eq!(
            "00000 0".parse(),
            Ok(DataItem {
                input: "00000".to_owned(),
                output: "0".to_owned()
            })
        );
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
            "0.981136 0.369132 0.498354 0.067417 0.422276 0.803662 1".parse::<DataItem>(),
            Ok(DataItem {
                input: "100001".to_owned(),
                output: "1".to_owned()
            })
        );
    }

    #[test]
    fn test_num_digits_width() {
        let data_item = DataItem::from_str("00001 1").expect("data item input is invalid");
        assert_eq!(data_item.width(), 5);

        let data_item =
            DataItem::from_str("0.981136 0.369132 0.498354 0.067417 0.422276 0.803662 1")
                .expect("data item input is invalid");
        assert_eq!(data_item.width(), 6);
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
        let data_item =
            DataItem::from_str("0.981136 0.369132 0.498354 0.067417 0.422276 0.803662 1")
                .expect("data item input is invalid");
        assert_eq!(data_item.char_at(0), Some('9'));
        assert_eq!(data_item.char_at(37), None);
    }
}
