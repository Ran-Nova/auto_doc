use auto_doc::auto_doc;

#[auto_doc(members = true)]
impl<T> MissingExample<T> {
    const ANSWER: u8 = 42;

    fn hello() {}
}

struct MissingExample<T>(T);

fn main() {}
