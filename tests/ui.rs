#[test]
fn trybuild() {
    let t = trybuild::TestCases::new();

    #[cfg(not(feature = "advanced"))]
    {
        t.pass("tests/ui/default/pass/*.rs");
        t.compile_fail("tests/ui/default/fail/*.rs");
    }

    #[cfg(feature = "advanced")]
    {
        t.pass("tests/ui/advanced/pass/*.rs");
        t.compile_fail("tests/ui/advanced/fail/*.rs");
    }
}
