use auto_doc::auto_doc;

trait Trait {
    const ANSWER: u8;
    fn hello();
}

#[auto_doc(members = true, member_path = "docs/{type}/{member}.md")]
impl<T> Trait for Example<T> {
    const ANSWER: u8 = 42;

    fn hello() {}
}

struct Example<T>(T);

fn main() {}
