use csv::Writer;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TryIntoCsvError {
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    CsvError(#[from] csv::Error),
    #[error("csv IntoInnerError")]
    CsvIntoInnerError,
}

pub trait TryIntoCsv<T>
where
    Self: std::iter::IntoIterator<Item = T> + Clone,
    T: Serialize,
{
    fn try_into_csv(&self) -> Result<String, TryIntoCsvError> {
        let mut csv_writer = Writer::from_writer(vec![]);
        for item in self.clone().into_iter() {
            csv_writer.serialize(item)?;
        }
        let text = String::from_utf8(csv_writer.into_inner().map_err(|_| TryIntoCsvError::CsvIntoInnerError)?)?;

        Ok(text)
    }
}
