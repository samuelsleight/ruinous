use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor},
    path::Path,
};

use ruinous_util::span::Span;

use crate::reader::{error::FileError, CharReader};

use self::state::State;

pub use self::error::Error;

pub mod state;

mod error;

pub struct Lexer<R> {
    reader: CharReader<R>,
}

impl Lexer<BufReader<File>> {
    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, FileError> {
        let reader = CharReader::with_file(path)?;
        Ok(Self { reader })
    }
}

impl<'a> Lexer<Cursor<&'a str>> {
    pub fn with_str(input: &'a str) -> Self {
        let reader = CharReader::with_str(input);
        Self { reader }
    }
}

impl<R: BufRead> Lexer<R> {
    pub fn lex<S: State, Callback: FnMut(Span<S::Token>)>(
        self,
        mut state: S,
        mut callback: Callback,
    ) -> Result<(), Error<S>> {
        self.reader
            .read(|span| state.process(span, &mut callback))?;
        state.finish().map_err(Error::LexError)
    }
}
