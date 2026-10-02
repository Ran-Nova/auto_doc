mod native_docs {
    #[doc = include_str!("../../../../docs/example.md")]
    #[macro_export]
    macro_rules! native_doc_macro {
        () => {};
    }
}

mod consumer {
    #[allow(unused_imports)]
    use crate::native_doc_macro;

    pub fn run() {
        native_doc_macro!();
    }
}

fn main() {
    consumer::run();
}
