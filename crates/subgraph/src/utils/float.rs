use alloy::primitives::fixed_bytes;
use rain_math_float::Float;

pub const FMIN: Float = Float::from_raw(fixed_bytes!(
    "7fffffff80000000000000000000000000000000000000000000000000000000"
));
pub const NEG7: Float = Float::from_raw(fixed_bytes!(
    "00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffff9"
));
pub const NEG6: Float = Float::from_raw(fixed_bytes!(
    "00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffa"
));
pub const NEG5: Float = Float::from_raw(fixed_bytes!(
    "00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffb"
));
pub const NEG2: Float = Float::from_raw(fixed_bytes!(
    "00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
));
pub const NEG1: Float = Float::from_raw(fixed_bytes!(
    "00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
));
pub const NEG0_5: Float = Float::from_raw(fixed_bytes!(
    "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb"
));
pub const F0: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000000"
));
pub const F0_5: Float = Float::from_raw(fixed_bytes!(
    "ffffffff00000000000000000000000000000000000000000000000000000005"
));
pub const F1: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000001"
));
pub const F1_5: Float = Float::from_raw(fixed_bytes!(
    "ffffffff0000000000000000000000000000000000000000000000000000000f"
));
pub const F2: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000002"
));
pub const F3: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000003"
));
pub const F4: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000004"
));
pub const F5: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000005"
));
pub const F6: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000006"
));
pub const F7: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000007"
));
pub const F10: Float = Float::from_raw(fixed_bytes!(
    "000000000000000000000000000000000000000000000000000000000000000a"
));
pub const F12: Float = Float::from_raw(fixed_bytes!(
    "000000000000000000000000000000000000000000000000000000000000000c"
));
pub const F15: Float = Float::from_raw(fixed_bytes!(
    "000000000000000000000000000000000000000000000000000000000000000f"
));
pub const F20: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000014"
));
pub const F25: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000019"
));
pub const F30: Float = Float::from_raw(fixed_bytes!(
    "000000000000000000000000000000000000000000000000000000000000001e"
));
pub const F35: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000023"
));
pub const F50: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000032"
));
pub const F100: Float = Float::from_raw(fixed_bytes!(
    "0000000000000000000000000000000000000000000000000000000000000064"
));
pub const F1000: Float = Float::from_raw(fixed_bytes!(
    "00000000000000000000000000000000000000000000000000000000000003e8"
));
pub const FMAX: Float = Float::from_raw(fixed_bytes!(
    "7fffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffff"
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_binary_representations() {
        // Verify that hard-coded binary representations match original values
        // TODO: Uncomment this when we have Float::min_negative
        // assert!(FMIN
        //     .eq(Float::pack_lossless(I224::MIN, i32::MAX).unwrap())
        //     .unwrap());
        assert!(NEG7.eq(Float::parse("-7".to_string()).unwrap()).unwrap());
        assert!(NEG6.eq(Float::parse("-6".to_string()).unwrap()).unwrap());
        assert!(NEG5.eq(Float::parse("-5".to_string()).unwrap()).unwrap());
        assert!(NEG2.eq(Float::parse("-2".to_string()).unwrap()).unwrap());
        assert!(NEG1.eq(Float::parse("-1".to_string()).unwrap()).unwrap());
        assert!(NEG0_5
            .eq(Float::parse("-0.5".to_string()).unwrap())
            .unwrap());
        assert!(F0.eq(Float::parse("0".to_string()).unwrap()).unwrap());
        assert!(F0_5.eq(Float::parse("0.5".to_string()).unwrap()).unwrap());
        assert!(F1.eq(Float::parse("1".to_string()).unwrap()).unwrap());
        assert!(F1_5.eq(Float::parse("1.5".to_string()).unwrap()).unwrap());
        assert!(F2.eq(Float::parse("2".to_string()).unwrap()).unwrap());
        assert!(F3.eq(Float::parse("3".to_string()).unwrap()).unwrap());
        assert!(F4.eq(Float::parse("4".to_string()).unwrap()).unwrap());
        assert!(F5.eq(Float::parse("5".to_string()).unwrap()).unwrap());
        assert!(F6.eq(Float::parse("6".to_string()).unwrap()).unwrap());
        assert!(F7.eq(Float::parse("7".to_string()).unwrap()).unwrap());
        assert!(F10.eq(Float::parse("10".to_string()).unwrap()).unwrap());
        assert!(F12.eq(Float::parse("12".to_string()).unwrap()).unwrap());
        assert!(F15.eq(Float::parse("15".to_string()).unwrap()).unwrap());
        assert!(F20.eq(Float::parse("20".to_string()).unwrap()).unwrap());
        assert!(F25.eq(Float::parse("25".to_string()).unwrap()).unwrap());
        assert!(F30.eq(Float::parse("30".to_string()).unwrap()).unwrap());
        assert!(F35.eq(Float::parse("35".to_string()).unwrap()).unwrap());
        assert!(F50.eq(Float::parse("50".to_string()).unwrap()).unwrap());
        assert!(F100.eq(Float::parse("100".to_string()).unwrap()).unwrap());
        assert!(F1000.eq(Float::parse("1000".to_string()).unwrap()).unwrap());
        // TODO: Uncomment this when we have Float::max_positive
        // assert!(FMAX
        //     .eq(Float::pack_lossless(I224::MAX, i32::MAX).unwrap())
        //     .unwrap());
    }
}
