use std::path::Path;

use ruinous_util::span::Span;

use self::{reader::CharReader, state::State};

pub use self::error::Error;

pub mod reader;
pub mod state;

mod error;

pub struct Lexer;

impl Lexer {
    pub fn lex_file<P: AsRef<Path>, S: State, Callback: FnMut(Span<S::Token>)>(
        path: P,
        mut state: S,
        mut callback: Callback,
    ) -> Result<(), Error<S>> {
        let reader = CharReader::from_file(path)?;
        reader.read(|span| state.process(span, &mut callback))?;
        state.finish().map_err(Error::LexError)
    }
}
