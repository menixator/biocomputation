use crate::dataitem::{DataItem, DataItemParseError};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

lazy_static! {
    static ref HEADER_REGEX: Regex = Regex::new(r##"^(\d+) rows x (\d+) variables"##).unwrap();
}

/// A dataset is a container contains a list of `DataItem`s
#[derive(Clone, PartialEq, Debug)]
pub struct DataSet(Vec<DataItem>);

impl DataSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, data_item: DataItem) -> Result<(), DataSetError> {
        if self.0.len() > 0 {
            if data_item.is_binary() != self.0[0].is_binary() {
                return Err(DataSetError::HeterogenousData);
            }

            if data_item.width() != self.0[0].width() {
                return Err(DataSetError::LengthMismatch);
            }
        }
        self.0.push(data_item);
        Ok(())
    }

    pub fn split_at_percentage(
        self,
        percentage: usize,
    ) -> Result<(DataSet, DataSet), DataSetError> {
        if percentage > 100 {
            return Err(DataSetError::InvalidPercentage);
        }
        let percentage = percentage as f64;
        let split_index = (percentage / 100.0) * self.0.len() as f64;
        let split_index = split_index as usize;

        let mut first_vec = Vec::with_capacity(split_index);
        let mut second_vec = Vec::with_capacity(self.0.len() - split_index);

        for (index, data_item) in self.0.into_iter().enumerate() {
            if index < split_index {
                first_vec.push(data_item);
            } else {
                second_vec.push(data_item);
            }
        }
        Ok((DataSet(first_vec), DataSet(second_vec)))
    }
}

#[derive(Error, Clone, PartialEq, Debug)]
pub enum DataSetError {
    #[error("found different kinds of data")]
    HeterogenousData,

    #[error("length is not consistent")]
    LengthMismatch,

    #[error("percentage should be between 0 and 100")]
    InvalidPercentage,
}

#[derive(Error, Clone, PartialEq, Debug)]
pub enum DataSetParseError {
    #[error("an io error occured")]
    IoError(std::io::ErrorKind),

    #[error("failed to parse data item due to {source}")]
    DataItemParseError {
        line_number: usize,
        #[source]
        source: DataItemParseError,
    },

    #[error("failed to add item to data set due to: {source}")]
    DataSetError {
        line_number: usize,
        #[source]
        source: DataSetError,
    },
}

impl DataSet {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<DataSet, DataSetParseError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|err| DataSetParseError::IoError(err.kind()))?;
        let reader = BufReader::new(file);

        let mut data_set = DataSet(vec![]);

        for (line_number, line) in reader.lines().enumerate() {
            let line = line.map_err(|err| DataSetParseError::IoError(err.kind()))?;
            if line_number == 0 {
                if !HEADER_REGEX.is_match(&line) { /**/ }
                continue;
            }
            let data_item: DataItem =
                line.parse()
                    .map_err(|err| DataSetParseError::DataItemParseError {
                        source: err,
                        line_number,
                    })?;
            data_set
                .push(data_item)
                .map_err(|source| DataSetParseError::DataSetError {
                    line_number,
                    source,
                })?;
        }
        Ok(data_set)
    }
}
