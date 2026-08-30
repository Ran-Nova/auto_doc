use auto_doc::auto_doc;

#[auto_doc]
impl<T> Trait for Example<T> {}

trait Trait {}

struct Example<T>(T);

fn main() {}
