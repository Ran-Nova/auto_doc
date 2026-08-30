use std::{fs, path::PathBuf};

fn prepare_trybuild_docs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = root.join("target/tests/trybuild/auto_doc");

    let docs_dir = workspace.join("docs");

    fs::create_dir_all(docs_dir.join("Example")).unwrap();

    fs::write(
        docs_dir.join("example.md"),
        "# Example\n\nThis is a default-mode example document.\n",
    )
    .unwrap();

    fs::write(
        docs_dir.join("Example/ANSWER.md"),
        "# ANSWER\n\nThe answer for the example constant.\n",
    )
    .unwrap();

    fs::write(
        docs_dir.join("Example/hello.md"),
        "# hello\n\nThis is the hello method docs.\n",
    )
    .unwrap();
}

#[test]
fn trybuild() {
    prepare_trybuild_docs();

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
