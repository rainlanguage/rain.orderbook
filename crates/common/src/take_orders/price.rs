use crate::raindex_client::RaindexError;
use rain_math_float::Float;
use std::cmp::Ordering;

pub fn cmp_float(a: &Float, b: &Float) -> Result<Ordering, RaindexError> {
    if a.lt(*b)? {
        Ok(Ordering::Less)
    } else if a.gt(*b)? {
        Ok(Ordering::Greater)
    } else {
        Ok(Ordering::Equal)
    }
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use rain_math_float::Float;
    use std::cmp::Ordering;

    #[test]
    fn test_cmp_float_less() {
        let a = Float::parse("1".to_string()).unwrap();
        let b = Float::parse("2".to_string()).unwrap();
        assert_eq!(cmp_float(&a, &b).unwrap(), Ordering::Less);
    }

    #[test]
    fn test_cmp_float_greater() {
        let a = Float::parse("2".to_string()).unwrap();
        let b = Float::parse("1".to_string()).unwrap();
        assert_eq!(cmp_float(&a, &b).unwrap(), Ordering::Greater);
    }

    #[test]
    fn test_cmp_float_equal() {
        let a = Float::parse("1".to_string()).unwrap();
        let b = Float::parse("1".to_string()).unwrap();
        assert_eq!(cmp_float(&a, &b).unwrap(), Ordering::Equal);
    }
}
