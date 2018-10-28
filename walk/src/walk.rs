use err::Error;
use std::fs::{self, read_dir};
use std::path::{Path, PathBuf};

macro_rules! try_opt_res {
    ($e: expr) => {
        match $e {
            Ok(v) => v,
            Err(v) => return Some(Err(Error::from(v))),
        }
    };
}

pub trait MetadataTrait {
    fn is_dir(&self) -> bool;
}
pub trait DirEntryTrait {
    type Path;
    fn path(&self) -> Self::Path;
}
pub trait ReadDirTrait {}

struct ReadDirWrapper(fs::ReadDir);
struct MetadataWrapper(fs::Metadata);
struct DirEntryWrapper(fs::DirEntry);

impl ReadDirTrait for ReadDirWrapper {}

impl Iterator for ReadDirWrapper {
    type Item = Result<DirEntryWrapper, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r| r.map(|r2| DirEntryWrapper(r2)).map_err(Error::from))
    }
}

impl MetadataTrait for MetadataWrapper {
    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }
}

impl DirEntryTrait for DirEntryWrapper {
    type Path = PathBuf;
    fn path(&self) -> Self::Path {
        self.0.path()
    }
}

pub trait FileSystem {
    type FSPath: std::fmt::Debug;
    type DirEntry: DirEntryTrait<Path = Self::FSPath>;
    type ReadDir: ReadDirTrait + Iterator<Item = Result<Self::DirEntry, Error>>;
    type Metadata: MetadataTrait;

    fn is_dir(&self, path: &Self::FSPath) -> bool {
        self.metadata(path).map(|a| a.is_dir()).unwrap_or(false)
    }

    fn read_dir(&self, path: &Self::FSPath) -> Result<Self::ReadDir, Error>;
    fn metadata(&self, path: &Self::FSPath) -> Result<Self::Metadata, Error>;
}

struct LinuxFS {}

impl FileSystem for LinuxFS {
    type FSPath = PathBuf;
    type DirEntry = DirEntryWrapper;
    type ReadDir = ReadDirWrapper;
    type Metadata = MetadataWrapper;

    fn read_dir(&self, path: &Self::FSPath) -> Result<Self::ReadDir, Error> {
        Ok(ReadDirWrapper(read_dir(path)?))
    }

    fn metadata(&self, path: &Self::FSPath) -> Result<Self::Metadata, Error> {
        Ok(MetadataWrapper(fs::metadata(path)?))
    }
}

pub struct Walk<T: FileSystem> {
    path_stack: Vec<T::FSPath>,
    dir_entries_stack: Vec<T::FSPath>,
    fs: T,
}

impl<T: FileSystem> Walk<T> {
    pub fn new(fs: T, path: T::FSPath) -> Result<Walk<T>, Error> {
        let mut path_stack = vec![];
        let mut dir_entries = vec![];
        if fs.is_dir(&path) {
            path_stack.push(path)
        } else {
            dir_entries.push(path)
        }

        Ok(Walk {
            path_stack: path_stack,
            dir_entries_stack: dir_entries,
            fs: fs,
        })
    }
}

impl<T: FileSystem> Iterator for Walk<T> {
    type Item = Result<T::FSPath, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.dir_entries_stack.len() > 0 {
            return self.dir_entries_stack.pop().map(|e| Ok(e));
        } else {
            if let Some(next_path) = self.path_stack.pop() {
                for entry in try_opt_res!(self.fs.read_dir(&next_path)) {
                    let entry = try_opt_res!(entry);
                    let path = entry.path();

                    if self.fs.is_dir(&path) {
                        self.path_stack.push(path)
                    } else {
                        self.dir_entries_stack.push(path)
                    }
                }
                return self.next();
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use walk::*;
    #[test]
    fn can_read_dcir() {
        let fs = LinuxFS {};
        let path = PathBuf::from("/opt/libhdfs3");
        for p in Walk::new(fs, path).unwrap() {
            let p = p.unwrap();
            println!("{:?}", p);
        }
    }
}
