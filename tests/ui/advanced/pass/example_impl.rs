use auto_doc::auto_doc;

#[auto_doc(members = true)]
impl Example {
    const ANSWER: u8 = 42;

    fn hello() {}
}

struct Example;

fn main() {}
