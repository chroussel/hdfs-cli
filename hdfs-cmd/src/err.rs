#[derive(Debug)]
pub enum Error {
    WalkError(walk::err::Error),
    HdfsError(hdfs::err::Error),
    IoError(std::io::Error),
    SerializationError(toml::ser::Error),
    NoHome,
}

impl From<hdfs::err::Error> for Error {
    fn from(err: hdfs::err::Error) -> Error {
        Error::HdfsError(err)
    }
}

impl From<walk::err::Error> for Error {
    fn from(err: walk::err::Error) -> Error {
        Error::WalkError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Error {
        Error::SerializationError(err)
    }
}
