use err;
use std::fs;
use std::path::PathBuf;
use walk::{DirEntryTrait, FileSystem, MetadataTrait};

pub struct ReadDirWrapper(fs::ReadDir);
pub struct MetadataWrapper(fs::Metadata);
pub struct DirEntryWrapper(fs::DirEntry);

impl Iterator for ReadDirWrapper {
    type Item = Result<DirEntryWrapper, err::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r| r.map(DirEntryWrapper).map_err(err::Error::from))
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
        self.0.path().is_dir()
    }
}

#[derive(Default)]
pub struct LinuxFS {}

impl FileSystem for LinuxFS {
    type Error = err::Error;
    type DirEntry = DirEntryWrapper;
    type ReadDir = ReadDirWrapper;
    type Metadata = MetadataWrapper;

    fn exists(&self, path: &PathBuf) -> bool {
        fs::metadata(path).is_ok()
    }

    fn read_dir(&self, path: &PathBuf) -> Result<Self::ReadDir, Self::Error> {
        Ok(ReadDirWrapper(fs::read_dir(path)?))
    }

    fn metadata(&self, path: &PathBuf) -> Result<Self::Metadata, Self::Error> {
        Ok(MetadataWrapper(fs::metadata(path)?))
    }
}
