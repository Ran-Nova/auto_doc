use auto_doc::auto_doc;

#[auto_doc(members = true)]
trait ExampleTrait {
    const ANSWER: u8;

    type Value;

    fn hello();
}

#[auto_doc(members = true)]
enum ExampleEnum {
    First,
    Second,
}

fn main() {}
