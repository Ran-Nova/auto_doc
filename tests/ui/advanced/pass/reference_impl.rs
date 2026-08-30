use auto_doc::auto_doc;

trait Trait {
    fn hello();
}

#[auto_doc(members = true, member_path = "docs/{type}/{member}.md")]
impl Trait for &Example {
    fn hello() {}
}

struct Example;

fn main() {}
