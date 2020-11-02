#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/v1.0.6a/01-ast-deserialize.rs");
}
