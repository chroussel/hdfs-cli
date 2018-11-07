#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    PathConversionError(std::ffi::OsString),
    PatternError(glob::PatternError),
    NoPathDefined,
    PathFormatError,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(err: std::ffi::OsString) -> Self {
        Error::PathConversionError(err)
    }
}

impl From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Self {
        Error::PatternError(err)
    }
}
