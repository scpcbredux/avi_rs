use std::io;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum AviError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    String(#[from] std::string::FromUtf8Error),
    #[error("Not a RIFF file.")]
    NotRiff,
    #[error("Not an AVI file.")]
    NotAvi,
    #[error("No more frames")]
    NoMoreFrames,
    #[error("Pointer out of range")]
    PointerOutOfRange,
}

pub type Result<T> = std::result::Result<T, AviError>;
