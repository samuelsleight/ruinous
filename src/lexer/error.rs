use std::{
    fmt::Debug,
    io::{Read, Seek},
};

use ruinous_util::error::{context::ErrorProvider, writer::ErrorWriter};

use crate::reader::error::FileError;

use super::state::State;

pub enum Error<S: State> {
    FileError(FileError),
    LexError(S::Error),
}

impl<S: State> From<FileError> for Error<S> {
    fn from(error: FileError) -> Self {
        Error::FileError(error)
    }
}

impl<S: State> Debug for Error<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileError(error) => f.debug_tuple("FileError").field(&error).finish(),
            Error::LexError(error) => f.debug_tuple("LexError").field(&error).finish(),
        }
    }
}

impl<S: State> ErrorProvider for Error<S> {
    fn write_errors<R: Read + Seek>(&self, writer: &mut ErrorWriter<R>) -> std::fmt::Result {
        match self {
            Error::FileError(error) => error.write_errors(writer),
            Error::LexError(error) => error.write_errors(writer),
        }
    }
}

impl ErrorProvider for FileError {
    fn write_errors<R: Read + Seek>(&self, writer: &mut ErrorWriter<R>) -> std::fmt::Result {
        match self {
            FileError::FileOpen { file, source } => writer.error(
                None,
                &format!("Unable to open file `{}`: {}", file.display(), source),
            ),
            FileError::FileRead { file, source } => writer.error(
                None,
                &format!("Unable to read file `{}`: {}", file.display(), source),
            ),
        }
    }
}
