use auto_doc::auto_doc;

#[auto_doc(members = true, member_path = "docs/{type}/{member}.md")]
impl<T> Example<T> {
    const ANSWER: u8 = 42;

    fn hello() {}
}

struct Example<T>(T);

fn main() {}
