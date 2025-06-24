pub(crate) mod api;
pub(crate) mod cli;
pub(crate) mod config;

pub(crate) type IoResult<T> = std::io::Result<T>;
pub(crate) type BruleeError = Box<dyn std::error::Error>;
pub(crate) type BruleeResult<T> = Result<T, BruleeError>;
