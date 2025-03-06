#[test]
pub fn pass() {
    // to update the tests "*.expanded.rs" delete them
    // and run cargo test to regenerate them
    macrotest::expand("tests/pass/*.rs");
}

#[test]
fn fail() {
    // to update the tests "*.stderr" set env TRYBUILD=overwrite
    // and run cargo test
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
}
