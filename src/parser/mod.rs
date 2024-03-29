use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor},
    path::Path,
};

use self::state::State as ParserState;
use crate::{
    lexer::{state::State as LexerState, Lexer},
    reader::error::FileError,
};

pub use self::error::{Error, ParseErrors};

pub mod state;

mod error;

pub struct Parser<R> {
    lexer: Lexer<R>,
}

impl Parser<BufReader<File>> {
    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, FileError> {
        let lexer = Lexer::with_file(path)?;
        Ok(Self { lexer })
    }
}

impl<'a> Parser<Cursor<&'a str>> {
    pub fn with_str(input: &'a str) -> Self {
        let lexer = Lexer::with_str(input);
        Self { lexer }
    }
}

impl<R: BufRead> Parser<R> {
    pub fn parse<LS: LexerState, PS: ParserState<LS::Token>>(
        self,
        lexer: LS,
        mut parser: PS,
    ) -> Result<PS::Ast, Error<LS, PS>> {
        self.lexer
            .lex(lexer, |token| parser.process(token))
            .map_err(Error::LexError)?;

        parser.finish().map_err(Error::ParseErrors)
    }
}
