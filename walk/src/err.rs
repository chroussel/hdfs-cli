#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    PathConversionError(std::ffi::OsString),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        return Error::IoError(err);
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(err: std::ffi::OsString) -> Self {
        return Error::PathConversionError(err);
    }
}
