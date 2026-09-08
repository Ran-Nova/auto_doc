use auto_doc::auto_doc;

struct Example;

#[auto_doc(members = true, member_path = "docs/{type}/{member}.md")]
impl Example {
    const ANSWER: u8 = 42;

    fn hello() {}
}

fn main() {}
