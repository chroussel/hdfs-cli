use err::Error;
use std::path::PathBuf;
use walk::walk::{DirEntryTrait, FileSystem, MetadataTrait};

pub struct HdfsFileSystem<'a>(&'a hdfs::hdfs::HDFileSystem);

impl<'a> HdfsFileSystem<'a> {
    pub fn new(fs: &'a hdfs::hdfs::HDFileSystem) -> HdfsFileSystem {
        HdfsFileSystem(fs)
    }
}

pub struct ReadDirWrapper(hdfs::hdfs::ReadDir);
pub struct MetadataWrapper(hdfs::hdfs::DirEntry);
pub struct DirEntryWrapper(hdfs::hdfs::DirEntry);

impl Iterator for ReadDirWrapper {
    type Item = Result<DirEntryWrapper, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| Ok(DirEntryWrapper(entry)))
    }
}

impl MetadataTrait for MetadataWrapper {
    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }
}

impl DirEntryTrait for DirEntryWrapper {
    fn path(&self) -> PathBuf {
        self.0.path()
    }

    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }
}

impl<'a> FileSystem for HdfsFileSystem<'a> {
    type Error = Error;
    type DirEntry = DirEntryWrapper;
    type ReadDir = ReadDirWrapper;
    type Metadata = MetadataWrapper;

    fn exists(&self, path: &PathBuf) -> bool {
        self.0.exists(path).unwrap_or(false)
    }

    fn read_dir(&self, path: &PathBuf) -> Result<Self::ReadDir, Self::Error> {
        Ok(ReadDirWrapper(self.0.list_directory(path)?))
    }
    fn metadata(&self, path: &PathBuf) -> Result<Self::Metadata, Self::Error> {
        Ok(MetadataWrapper(self.0.path_info(path)?))
    }
}
