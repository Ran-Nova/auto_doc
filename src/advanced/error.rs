use proc_macro2::Span;
use syn::Error as SynError;

#[derive(Debug)]
pub(crate) enum AdvancedError {
    Attribute(String),
    Syntax(SynError),
    InvalidConfiguration(&'static str),
}

impl AdvancedError {
    pub(crate) fn into_syn_error(self) -> SynError {
        match self {
            Self::Attribute(message) => SynError::new(Span::call_site(), message),
            Self::Syntax(error) => error,
            Self::InvalidConfiguration(message) => SynError::new(Span::call_site(), message),
        }
    }
}

impl From<SynError> for AdvancedError {
    fn from(error: SynError) -> Self {
        Self::Syntax(error)
    }
}
