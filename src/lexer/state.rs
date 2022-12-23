use ruinous_util::{error::context::ErrorProvider, span::Span};

pub enum Continuation {
    Consume,
    Peek,
}

pub trait State {
    type Token;
    type Error: ErrorProvider;

    fn process<Callback: FnMut(Span<Self::Token>)>(
        &mut self,
        input: Span<char>,
        callback: &mut Callback,
    ) -> Continuation;

    fn finish(self) -> Result<(), Self::Error>;
}
