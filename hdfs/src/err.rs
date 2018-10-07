use std::io;
use util::chars_to_str;
use native;

#[derive(Debug)]
pub enum HdfsErr {
    Unknown,
    FileNotFound(String),
    DirectoryNotFound(String),
    InvalidXmlFile(String),
    HdfsError(String),
    Io(io::Error),
    MissingConfig(String),
    ErrorCreatingBuilder,
    PathConversionError(String),
    NoError()
}

impl From<io::Error> for HdfsErr {
    fn from(err: io::Error) -> HdfsErr {
        HdfsErr::Io(err)
    }
}

impl HdfsErr {
    pub fn get_last_error() -> HdfsErr {
        let os_error = io::Error::last_os_error();
        HdfsErr::Io(os_error)
    }

    pub fn get_last_hdfs_error() -> HdfsErr {
        let hdfs_error_raw = chars_to_str(unsafe { native::hdfsGetLastError()});
        if hdfs_error_raw == "Success" {
            HdfsErr::NoError()
        } else {
            HdfsErr::HdfsError(hdfs_error_raw.to_owned())
        }
    }
}