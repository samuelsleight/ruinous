use std::fmt::Debug;

use ruinous_util::error::{context::ErrorProvider, writer::ErrorWriter};

use super::state::State as ParserState;
use crate::{
    lexer::{state::State as LexerState, Error as LexError},
    reader::error::FileError,
};

pub enum Error<L: LexerState, P: ParserState<L::Token>> {
    LexError(LexError<L>),
    ParseErrors(ParseErrors<P::Error>),
}

pub struct ParseErrors<E: ErrorProvider> {
    errors: Vec<E>,
}

impl<E: ErrorProvider> From<Vec<E>> for ParseErrors<E> {
    fn from(errors: Vec<E>) -> Self {
        Self { errors }
    }
}

impl<L: LexerState, P: ParserState<L::Token>> From<FileError> for Error<L, P> {
    fn from(error: FileError) -> Self {
        Self::LexError(error.into())
    }
}

impl<L: LexerState, P: ParserState<L::Token>> Debug for Error<L, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LexError(error) => f.debug_tuple("LexError").field(&error).finish(),
            Error::ParseErrors(error) => f.debug_tuple("ParseErrors").field(&error).finish(),
        }
    }
}

impl<E: ErrorProvider> Debug for ParseErrors<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseErrors")
            .field("errors", &self.errors)
            .finish()
    }
}

impl<L: LexerState, P: ParserState<L::Token>> ErrorProvider for Error<L, P> {
    fn write_errors<R: std::io::Read + std::io::Seek>(
        &self,
        writer: &mut ErrorWriter<R>,
    ) -> std::fmt::Result {
        match self {
            Error::LexError(error) => error.write_errors(writer)?,
            Error::ParseErrors(errors) => {
                for error in &errors.errors {
                    error.write_errors(writer)?
                }
            }
        }

        Ok(())
    }
}
