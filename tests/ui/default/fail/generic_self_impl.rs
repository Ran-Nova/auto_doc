use auto_doc::auto_doc;

#[auto_doc]
impl<T> Example<T> {}

struct Example<T>(T);

fn main() {}
