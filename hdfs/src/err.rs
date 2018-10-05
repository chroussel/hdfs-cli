use std::io;

#[derive(Debug)]
pub enum HdfsErr {
    Unknown,
    FileNotFound(String),
    DirectoryNotFound(String),
    InvalidXmlFile(String),
    CannotConnectToNameNode((String, String)),
    Io(io::Error),
    MissingConfig(String),
    ErrorCreatingBuilder,
}

impl From<io::Error> for HdfsErr {
    fn from(err: io::Error) -> HdfsErr {
        HdfsErr::Io(err)
    }
}
