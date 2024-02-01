use csv::Writer;
use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteCsvError {
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    CsvError(#[from] csv::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub trait WriteCsv<T>
where
    Self: std::iter::IntoIterator<Item = T> + Clone,
    T: Serialize,
{
    fn write_csv(&self, path: PathBuf) -> Result<(), WriteCsvError> {
        let mut csv_writer = Writer::from_path(path.as_path())?;

        for item in self.clone().into_iter() {
            csv_writer.serialize(item)?;
        }
        csv_writer.flush()?;

        Ok(())
    }
}
