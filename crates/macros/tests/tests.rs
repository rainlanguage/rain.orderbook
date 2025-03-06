#[test]
pub fn pass() {
    macrotest::expand("tests/pass/*.rs");
}

#[test]
fn fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
}
