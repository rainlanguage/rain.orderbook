use alloy::primitives::aliases::I224;
use rain_math_float::Float;

lazy_static::lazy_static! {
    pub static ref FMIN: Float = Float::pack_lossless(I224::MIN, i32::MIN).unwrap();
    pub static ref NEG7: Float = Float::parse("-7".to_string()).unwrap();
    pub static ref NEG6: Float = Float::parse("-6".to_string()).unwrap();
    pub static ref NEG5: Float = Float::parse("-5".to_string()).unwrap();
    pub static ref NEG2: Float = Float::parse("-2".to_string()).unwrap();
    pub static ref NEG1: Float = Float::parse("-1".to_string()).unwrap();
    pub static ref NEG0_5: Float = Float::parse("-0.5".to_string()).unwrap();
    pub static ref F0: Float = Float::parse("0".to_string()).unwrap();
    pub static ref F0_5: Float = Float::parse("0.5".to_string()).unwrap();
    pub static ref F1: Float = Float::parse("1".to_string()).unwrap();
    pub static ref F1_5: Float = Float::parse("1.5".to_string()).unwrap();
    pub static ref F2: Float = Float::parse("2".to_string()).unwrap();
    pub static ref F3: Float = Float::parse("3".to_string()).unwrap();
    pub static ref F4: Float = Float::parse("4".to_string()).unwrap();
    pub static ref F5: Float = Float::parse("5".to_string()).unwrap();
    pub static ref F6: Float = Float::parse("6".to_string()).unwrap();
    pub static ref F7: Float = Float::parse("7".to_string()).unwrap();
    pub static ref F10: Float = Float::parse("10".to_string()).unwrap();
    pub static ref F12: Float = Float::parse("12".to_string()).unwrap();
    pub static ref F15: Float = Float::parse("15".to_string()).unwrap();
    pub static ref F20: Float = Float::parse("20".to_string()).unwrap();
    pub static ref F25: Float = Float::parse("25".to_string()).unwrap();
    pub static ref F30: Float = Float::parse("30".to_string()).unwrap();
    pub static ref F35: Float = Float::parse("35".to_string()).unwrap();
    pub static ref F50: Float = Float::parse("50".to_string()).unwrap();
    pub static ref F100: Float = Float::parse("100".to_string()).unwrap();
    pub static ref F1000: Float = Float::parse("1000".to_string()).unwrap();
    pub static ref FMAX: Float = Float::pack_lossless(I224::MAX, i32::MAX).unwrap();
}
