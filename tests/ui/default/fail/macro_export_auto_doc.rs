use auto_doc::auto_doc;

#[auto_doc(path = "docs/example.md")]
#[macro_export]
macro_rules! exported_macro {
    () => {};
}

fn main() {}
