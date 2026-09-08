use auto_doc::auto_doc;

#[auto_doc(members = true)]
struct ExampleStruct {
    answer: u8,
    hello: (),
}

#[auto_doc(members = true)]
struct TupleStruct(u8);

fn main() {}
