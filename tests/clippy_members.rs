#![cfg(feature = "advanced")]

use auto_doc::auto_doc;

#[auto_doc(
    path = "docs/ClippyTrait.md",
    members = true,
    member_path = "docs/ClippyTrait/{member}.md"
)]
pub trait ClippyTrait {
    fn absolute_clean(&self) -> Result<(), std::io::Error>;
}

fn main() {}
