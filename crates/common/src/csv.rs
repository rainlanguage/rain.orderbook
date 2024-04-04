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
        let text = String::from_utf8(
            csv_writer
                .into_inner()
                .map_err(|_| TryIntoCsvError::CsvIntoInnerError)?,
        )?;

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Clone)]
    struct Person {
        name: String,
        age: u32,
    }
    impl TryIntoCsv<Person> for Vec<Person> {}
    #[test]
    fn test_try_into_csv() {
        let people = vec![
            Person {
                name: String::from("Alice"),
                age: 25,
            },
            Person {
                name: String::from("Bob"),
                age: 30,
            },
        ];

        let expected_csv = "name,age\nAlice,25\nBob,30\n";

        let result = people.try_into_csv();
        assert_eq!(result.unwrap(), expected_csv);
    }
}
