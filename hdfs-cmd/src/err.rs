#[derive(Debug)]
pub enum Error {
    WalkError(walk::err::Error),
    HdfsError(hdfs::err::Error),
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
