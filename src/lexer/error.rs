use std::{
    fmt::Debug,
    io::{Read, Seek},
    path::PathBuf,
};

use ruinous_util::error::{context::ErrorProvider, writer::ErrorWriter};

use super::state::State;

#[derive(Debug)]
pub enum FileError {
    FileOpen {
        file: PathBuf,
        source: std::io::Error,
    },
    FileRead {
        file: PathBuf,
        source: std::io::Error,
    },
}

pub enum Error<S: State> {
    FileError(FileError),
    LexError(S::Error),
}

impl FileError {
    #[must_use]
    pub fn file_open(path: PathBuf, source: std::io::Error) -> Self {
        FileError::FileOpen { file: path, source }
    }

    #[must_use]
    pub fn file_read(path: PathBuf, source: std::io::Error) -> Self {
        FileError::FileRead { file: path, source }
    }
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
