use std::path::Path;

use self::state::State as ParserState;
use crate::lexer::{state::State as LexerState, Lexer};

pub use self::error::{Error, ParseErrors};

pub mod state;

mod error;

pub struct Parser;

impl Parser {
    pub fn parse_file<P: AsRef<Path>, LS: LexerState, PS: ParserState<LS::Token>>(
        path: P,
        lexer: LS,
        mut parser: PS,
    ) -> Result<PS::Ast, Error<LS, PS>> {
        Lexer::lex_file(path, lexer, |token| parser.process(token)).map_err(Error::LexError)?;

        parser.finish().map_err(Error::ParseErrors)
    }
}
