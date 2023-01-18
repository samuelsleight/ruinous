use std::path::PathBuf;

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
