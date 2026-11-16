
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DedupError{
    #[error("i/o error encountered: {0}")]
    Io(#[from] std::io::Error),

    #[error("Directory traversal error:{0}")]
    WalkDir(#[from] ignore::Error),

    #[error("invalid path provided:{0}")]
    InvalidPath(String),

    #[error("operation aborted by user")]
    Aborted,

}

pub type Result<T> = std::result::Result<T, DedupError>;