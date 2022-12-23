use ruinous_util::{error::context::ErrorProvider, span::Span};

use super::error::ParseErrors;

pub trait State<Token>: Sized {
    type Ast;
    type Error: ErrorProvider;

    fn process(&mut self, token: Span<Token>);
    fn finish(self) -> Result<Self::Ast, ParseErrors<Self::Error>>;
}
