use std::{fs, path::PathBuf};

fn prepare_trybuild_docs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let docs_dir = root.join("docs");

    let files: Vec<(&str, &str)> = vec![
        (
            "example.md",
            "# Example\n\nThis is a default-mode example document.\n",
        ),
        (
            "example2.md",
            "# Example 2\n\nThis is a default-mode example2 document.\n",
        ),
        (
            "Example/ANSWER.md",
            "# ANSWER\n\nThe answer for the example constant.\n",
        ),
        (
            "Example/hello.md",
            "# hello\n\nThis is the hello method docs.\n",
        ),
        (
            "ExampleTrait.md",
            "# ExampleTrait\n\nTrait documentation.\n",
        ),
        (
            "ExampleTrait/ANSWER.md",
            "# ANSWER\n\nTrait constant documentation.\n",
        ),
        (
            "ExampleTrait/hello.md",
            "# hello\n\nTrait function documentation.\n",
        ),
        (
            "ExampleTrait/Value.md",
            "# Value\n\nTrait type documentation.\n",
        ),
        ("ExampleEnum.md", "# ExampleEnum\n\nEnum documentation.\n"),
        (
            "ExampleEnum/First.md",
            "# First\n\nFirst variant documentation.\n",
        ),
        (
            "ExampleEnum/Second.md",
            "# Second\n\nSecond variant documentation.\n",
        ),
        (
            "ExampleEnum/First/value.md",
            "# value\n\nFirst variant field documentation.\n",
        ),
        (
            "ExampleEnum/Third.md",
            "# Third\n\nThird variant documentation.\n",
        ),
        (
            "ExampleStruct.md",
            "# ExampleStruct\n\nStruct documentation.\n",
        ),
        (
            "ExampleStruct/answer.md",
            "# answer\n\nStruct field documentation.\n",
        ),
        (
            "ExampleStruct/hello.md",
            "# hello\n\nAnother struct field.\n",
        ),
        (
            "TupleStruct.md",
            "# TupleStruct\n\nTuple struct documentation.\n",
        ),
        (
            "source/SourceExample.md",
            "# Source example\n\nSource-path item documentation.\n",
        ),
        (
            "source/SourceExample/run.md",
            "# run\n\nSource-path member documentation.\n",
        ),
    ];

    for (rel_path, content) in files {
        let file_path = docs_dir.join(rel_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(file_path, content).unwrap();
    }
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
