use crate::{BigUintMath, MathError};
use alloy::primitives::U256;
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, Neg, Sub},
    str::FromStr,
};

/// Signed U256, is a wrapper struct around alloy's U256 with field
/// determining a negative/positive value to form a signed U256.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct SU256 {
    pub value: U256,
    pub is_neg: bool,
}

impl SU256 {
    /// Creates a new instance
    pub fn new(value: U256, is_neg: bool) -> SU256 {
        let is_neg_ = if value.is_zero() { false } else { is_neg };
        SU256 {
            value,
            is_neg: is_neg_,
        }
    }

    /// Returns true if this instance is negative else returns false
    pub fn is_negative(&self) -> bool {
        self.is_neg
    }

    /// Scales the value up by the given number of decimals
    pub fn scale_up(self, scale_up_by: u8) -> Result<SU256, MathError> {
        Ok(SU256 {
            value: self.value.scale_up(scale_up_by)?,
            is_neg: self.is_neg,
        })
    }

    /// Scales the value down by the given number of decimals
    pub fn scale_down(self, scale_down_by: u8) -> Result<SU256, MathError> {
        Ok(SU256 {
            value: self.value.scale_down(scale_down_by)?,
            is_neg: self.is_neg,
        })
    }

    /// Scales to 18 point fixed decimals
    pub fn scale_18(self, decimals: u8) -> Result<SU256, MathError> {
        Ok(SU256 {
            value: self.value.scale_18(decimals)?,
            is_neg: self.is_neg,
        })
    }

    /// Performs mulDiv operation
    pub fn mul_div(self, mul: SU256, div: SU256) -> Result<SU256, MathError> {
        Ok(SU256 {
            value: self.value.mul_div(mul.value, div.value)?,
            is_neg: (self.is_neg != mul.is_neg) != div.is_neg,
        })
    }

    /// Performs 18 fixed point mul operation
    pub fn mul_18(self, other: SU256) -> Result<SU256, MathError> {
        Ok(SU256 {
            value: self.value.mul_18(other.value)?,
            is_neg: self.is_neg != other.is_neg,
        })
    }

    /// Performs 18 fixed point div operation
    pub fn div_18(self, other: SU256) -> Result<SU256, MathError> {
        Ok(SU256 {
            value: self.value.div_18(other.value)?,
            is_neg: self.is_neg != other.is_neg,
        })
    }
}

impl Display for SU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // display as decimal integer
        let sign = if self.is_neg { "-" } else { "" };
        write!(f, "{}{}", sign, self.value)
    }
}

impl std::fmt::Debug for SU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // display as decimal integer
        let sign = if self.is_neg { "-" } else { "" };
        write!(f, "{}{}", sign, self.value)
    }
}

impl FromStr for SU256 {
    type Err = alloy::primitives::ruint::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let is_neg = s.starts_with('-');
        Ok(SU256 {
            #[allow(clippy::manual_strip)]
            value: U256::from_str(if is_neg { &s[1..] } else { s })?,
            is_neg,
        })
    }
}

impl Add for SU256 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let mut lhs = self;
        if lhs.is_neg == rhs.is_neg {
            lhs.value += rhs.value;
        } else {
            match lhs.value.cmp(&rhs.value) {
                Ordering::Greater => lhs.value -= rhs.value,
                Ordering::Less => {
                    lhs.value = rhs.value - lhs.value;
                    lhs.is_neg = !lhs.is_neg;
                }
                Ordering::Equal => {
                    lhs.value = U256::ZERO;
                    lhs.is_neg = false;
                }
            }
        }
        lhs
    }
}

impl Sub for SU256 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        if lhs.is_neg == rhs.is_neg {
            match lhs.value.cmp(&rhs.value) {
                Ordering::Greater => lhs.value -= rhs.value,
                Ordering::Less => {
                    lhs.value = rhs.value - lhs.value;
                    lhs.is_neg = !lhs.is_neg;
                }
                Ordering::Equal => {
                    lhs.value = U256::ZERO;
                    lhs.is_neg = false;
                }
            }
        } else {
            lhs.value += rhs.value;
        }
        lhs
    }
}

impl Neg for SU256 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(self.value, !self.is_neg)
    }
}

impl Ord for SU256 {
    fn clamp(self, _min: Self, _max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        self
    }
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_neg == other.is_neg {
            match self.value.cmp(&other.value) {
                Ordering::Greater => {
                    if self.is_neg {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                Ordering::Less => {
                    if self.is_neg {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                Ordering::Equal => Ordering::Equal,
            }
        } else if self.is_neg {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match self.cmp(&other) {
            Ordering::Greater => other,
            Ordering::Less => self,
            Ordering::Equal => self,
        }
    }
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match self.cmp(&other) {
            Ordering::Greater => self,
            Ordering::Less => other,
            Ordering::Equal => self,
        }
    }
}

impl PartialOrd for SU256 {
    fn ge(&self, other: &Self) -> bool {
        !matches!(self.cmp(other), Ordering::Less)
    }
    fn le(&self, other: &Self) -> bool {
        !matches!(self.cmp(other), Ordering::Greater)
    }
    fn gt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }
    fn lt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Less)
    }
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for SU256 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SU256 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(SU256DeserVisitor)
    }
}

struct SU256DeserVisitor;
impl<'de> Visitor<'de> for SU256DeserVisitor {
    type Value = SU256;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "an integer or hex string with a '-' prefix if negative"
        )
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        SU256::from_str(value).map_err(Error::custom)
    }
    fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
        SU256::from_str(&value).map_err(Error::custom)
    }

    fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_u16<E: Error>(self, v: u16) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_u32<E: Error>(self, v: u32) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_u128<E: Error>(self, v: u128) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }

    fn visit_i8<E: Error>(self, v: i8) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_i16<E: Error>(self, v: i16) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_i32<E: Error>(self, v: i32) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
    fn visit_i128<E: Error>(self, v: i128) -> Result<Self::Value, E> {
        Ok(SU256::from(v))
    }
}

impl From<u8> for SU256 {
    fn from(value: u8) -> Self {
        SU256 {
            value: U256::from(value),
            is_neg: false,
        }
    }
}
impl From<u16> for SU256 {
    fn from(value: u16) -> Self {
        SU256 {
            value: U256::from(value),
            is_neg: false,
        }
    }
}
impl From<u32> for SU256 {
    fn from(value: u32) -> Self {
        SU256 {
            value: U256::from(value),
            is_neg: false,
        }
    }
}
impl From<u64> for SU256 {
    fn from(value: u64) -> Self {
        SU256 {
            value: U256::from(value),
            is_neg: false,
        }
    }
}
impl From<u128> for SU256 {
    fn from(value: u128) -> Self {
        SU256 {
            value: U256::from(value),
            is_neg: false,
        }
    }
}
impl From<i8> for SU256 {
    fn from(value: i8) -> Self {
        SU256 {
            value: U256::from(if value.is_negative() {
                value.neg()
            } else {
                value
            } as u8),
            is_neg: value.is_negative(),
        }
    }
}
impl From<i16> for SU256 {
    fn from(value: i16) -> Self {
        SU256 {
            value: U256::from(if value.is_negative() {
                value.neg()
            } else {
                value
            } as u16),
            is_neg: value.is_negative(),
        }
    }
}
impl From<i32> for SU256 {
    fn from(value: i32) -> Self {
        SU256 {
            value: U256::from(if value.is_negative() {
                value.neg()
            } else {
                value
            } as u32),
            is_neg: value.is_negative(),
        }
    }
}
impl From<i64> for SU256 {
    fn from(value: i64) -> Self {
        SU256 {
            value: U256::from(if value.is_negative() {
                value.neg()
            } else {
                value
            } as u64),
            is_neg: value.is_negative(),
        }
    }
}
impl From<i128> for SU256 {
    fn from(value: i128) -> Self {
        SU256 {
            value: U256::from(if value.is_negative() {
                value.neg()
            } else {
                value
            } as u128),
            is_neg: value.is_negative(),
        }
    }
}

impl From<U256> for SU256 {
    fn from(value: U256) -> Self {
        SU256 {
            value,
            is_neg: false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_su256_from_str() {
        // positive
        for v in ["4567", "0x1234"] {
            let result = SU256::from_str(v).unwrap();
            assert_eq!(result.value, U256::from_str(v).unwrap());
            assert!(!result.is_neg);
        }

        // negative
        for v in ["-1234", "-0x987"] {
            let result = SU256::from_str(v).unwrap();
            assert_eq!(result.value, U256::from_str(&v[1..]).unwrap());
            assert!(result.is_neg);
        }
    }

    #[test]
    fn test_su256_fmt() {
        // positive display
        let result = format!("{}", SU256::from(123_u16));
        let expected = "123".to_string();
        assert_eq!(result, expected);
        let result = format!("{}", SU256::from(0x11_u32));
        let expected = "17".to_string();
        assert_eq!(result, expected);

        // positive debug
        let result = format!("{:?}", SU256::from(123_u16));
        let expected = "123".to_string();
        assert_eq!(result, expected);
        let result = format!("{:?}", SU256::from(0x11_u32));
        let expected = "17".to_string();
        assert_eq!(result, expected);

        // negative display
        let result = format!("{}", SU256::from(-123_i16));
        let expected = "-123".to_string();
        assert_eq!(result, expected);
        let result = format!("{}", SU256::from(-0x11_i32));
        let expected = "-17".to_string();
        assert_eq!(result, expected);

        // negative debug
        let result = format!("{:?}", SU256::from(-123_i16));
        let expected = "-123".to_string();
        assert_eq!(result, expected);
        let result = format!("{:?}", SU256::from(-0x11_i32));
        let expected = "-17".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_su256_add() {
        // positive + positive
        let result = SU256::from(123_u16) + SU256::from(0xa_u8);
        let expected = SU256::from(133_u16);
        assert_eq!(result, expected);

        // positive + negative
        let result = SU256::from(123_u16) + SU256::from(-0xa_i8);
        let expected = SU256::from(113_u16);
        assert_eq!(result, expected);

        // negative + negative
        let result = SU256::from(-123_i16) + SU256::from(-0xa_i8);
        let expected = SU256::from(-133_i16);
        assert_eq!(result, expected);

        // negative + positive
        let result = SU256::from(-123_i16) + SU256::from(0xa_u8);
        let expected = SU256::from(-113_i16);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_su256_sub() {
        // positive + positive
        let result = SU256::from(123_u16) - SU256::from(0xa_u8);
        let expected = SU256::from(113_u16);
        assert_eq!(result, expected);

        // positive + negative
        let result = SU256::from(123_u16) - SU256::from(-0xa_i8);
        let expected = SU256::from(133_u16);
        assert_eq!(result, expected);

        // negative + negative
        let result = SU256::from(-123_i16) - SU256::from(-0xa_i8);
        let expected = SU256::from(-113_i16);
        assert_eq!(result, expected);

        // negative + positive
        let result = SU256::from(-123_i16) - SU256::from(0xa_u8);
        let expected = SU256::from(-133_i16);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_su256_neg() {
        // positive
        let result = -SU256::from(123_u16);
        let expected = SU256::from(-123_i16);
        assert_eq!(result, expected);

        // negative
        let result = -SU256::from(-123_i16);
        let expected = SU256::from(123_u16);
        assert_eq!(result, expected);

        // zero
        let result = -SU256::from(0_i16);
        let expected = SU256::from(0_u16);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_su256_ord() {
        // positive > positive
        assert!(SU256::from(123_u16) > SU256::from(0xa_u8));

        // positive <= positive
        assert!(SU256::from(123_u16) <= SU256::from(222_u8));

        // positive == positive
        assert!(SU256::from(123_u16) == SU256::from(123_u8));

        // negative < negative
        assert!(SU256::from(-123_i16) < SU256::from(-0xa_i8));

        // negative >= negative
        assert!(SU256::from(-123_i16) >= SU256::from(-222_i16));

        // negative == negative
        assert!(SU256::from(-123_i16) == SU256::from(-123_i8));

        // positive > negative
        assert!(SU256::from(123_i16) > SU256::from(-123_i8));

        // negative <= positive
        assert!(SU256::from(-123_i16) <= SU256::from(123_i8));
    }

    #[test]
    fn test_su256_scale() {
        // scale up negative
        let result = SU256::from(-123_i16).scale_up(2).unwrap();
        let expected = SU256::from(-12300_i64);
        assert_eq!(result, expected);

        // scale up positive
        let result = SU256::from(123_i16).scale_up(2).unwrap();
        let expected = SU256::from(12300_i64);
        assert_eq!(result, expected);

        // scale down negative
        let result = SU256::from(-12300_i16).scale_down(2).unwrap();
        let expected = SU256::from(-123_i64);
        assert_eq!(result, expected);

        // scale down positive
        let result = SU256::from(12300_i16).scale_down(2).unwrap();
        let expected = SU256::from(123_i64);
        assert_eq!(result, expected);

        // scale 18 negative
        let result = SU256::from_str("-12300").unwrap().scale_18(2).unwrap();
        let expected = SU256::from_str("-123_000_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);

        // scale 18 positive
        let result = SU256::from_str("12300").unwrap().scale_18(2).unwrap();
        let expected = SU256::from_str("123_000_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_su256_fp_ops() {
        // mulDiv negative
        let result = SU256::from(-123_i16)
            .mul_div(SU256::from(4), SU256::from(2))
            .unwrap();
        let expected = SU256::from(-246_i64);
        assert_eq!(result, expected);

        // mulDiv positive
        let result = SU256::from(123_i16)
            .mul_div(SU256::from(4), SU256::from(2))
            .unwrap();
        let expected = SU256::from(246_i64);
        assert_eq!(result, expected);

        // mulDiv negative
        let result = SU256::from(-123_i16)
            .mul_div(SU256::from(-4), SU256::from(2))
            .unwrap();
        let expected = SU256::from(246_i64);
        assert_eq!(result, expected);

        // mulDiv positive
        let result = SU256::from(123_i16)
            .mul_div(SU256::from(-4), SU256::from(2))
            .unwrap();
        let expected = SU256::from(-246_i64);
        assert_eq!(result, expected);

        // mul 18 negative/negative
        let result = SU256::from_str("-123_000_000_000_000_000_000")
            .unwrap()
            .mul_18(SU256::from(-2))
            .unwrap();
        let expected = SU256::from(246_i64);
        assert_eq!(result, expected);

        // mul 18 positive/positive
        let result = SU256::from_str("123_000_000_000_000_000_000")
            .unwrap()
            .mul_18(SU256::from(2))
            .unwrap();
        let expected = SU256::from(246_i64);
        assert_eq!(result, expected);

        // mul 18 positive/negative
        let result = SU256::from_str("123_000_000_000_000_000_000")
            .unwrap()
            .mul_18(SU256::from(-2))
            .unwrap();
        let expected = SU256::from(-246_i64);
        assert_eq!(result, expected);

        // div 18 negative/negative
        let result = SU256::from(-123_i16).div_18(SU256::from(-2)).unwrap();
        let expected = SU256::from_str("61_500_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);

        // div 18 positive/positive
        let result = SU256::from(123_i16).div_18(SU256::from(2)).unwrap();
        let expected = SU256::from_str("61_500_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);

        // div 18 positive/negative
        let result = SU256::from(123_i16).div_18(SU256::from(-2)).unwrap();
        let expected = SU256::from_str("-61_500_000_000_000_000_000").unwrap();
        assert_eq!(result, expected);
    }
}
