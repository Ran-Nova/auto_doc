use auto_doc::auto_doc;

#[auto_doc(source = "source", members)]
struct SourceExample;

#[auto_doc(
    source = "source",
    members,
    member_path = "docs/{source}/{type}/{member}.md"
)]
impl SourceExample {
    fn run() {}
}

fn main() {}
