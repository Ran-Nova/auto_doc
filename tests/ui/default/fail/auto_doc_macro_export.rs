use auto_doc::auto_doc;

#[macro_export]
#[auto_doc(path = "docs/example.md")]
macro_rules! exported_macro {
    () => {};
}

fn main() {}
