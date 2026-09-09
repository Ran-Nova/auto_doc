#![cfg(feature = "advanced")]
#![deny(clippy::missing_errors_doc)]

use auto_doc::auto_doc;

#[doc = "# Errors"]
pub fn literal_doc() -> Result<(), std::io::Error> {
    Ok(())
}

#[auto_doc(path = "docs/ClippyTrait/absolute_clean.md")]
pub fn generated() -> Result<(), std::io::Error> {
    Ok(())
}

fn main() {}
