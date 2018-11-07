use native;
use std::io;
use std::path::PathBuf;
use util::chars_to_str;

#[derive(Debug)]
pub enum Error {
    Unknown,
    FileNotFound(String),
    DirectoryNotFound(PathBuf),
    InvalidXmlFile(String),
    HdfsError(String),
    Io(io::Error),
    MissingConfig(String),
    ErrorCreatingBuilder,
    PathConversionError(String),
    NoError(),
    InvalidPath(PathBuf),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl Error {
    pub fn get_last_error() -> Error {
        let os_error = io::Error::last_os_error();
        Error::Io(os_error)
    }

    pub fn get_last_hdfs_error() -> Error {
        let hdfs_error_raw = chars_to_str(unsafe { native::hdfsGetLastError() });
        if hdfs_error_raw == "Success" {
            Error::NoError()
        } else {
            Error::HdfsError(hdfs_error_raw.to_owned())
        }
    }
}
