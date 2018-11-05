use err;
use filter::*;
use std::fmt;
use std::fs::{self, read_dir};
use std::path::{Path, PathBuf};

macro_rules! try_opt_res {
    ($e: expr) => {
        match $e {
            Ok(v) => v,
            Err(v) => return Some(Err(v)),
        }
    };
}

pub trait MetadataTrait {
    fn is_dir(&self) -> bool;
}
pub trait DirEntryTrait {
    fn path(&self) -> PathBuf;
}

pub struct ReadDirWrapper(fs::ReadDir);
pub struct MetadataWrapper(fs::Metadata);
pub struct DirEntryWrapper(fs::DirEntry);

impl Iterator for ReadDirWrapper {
    type Item = Result<DirEntryWrapper, err::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r| r.map(|r2| DirEntryWrapper(r2)).map_err(err::Error::from))
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
}

pub trait FileSystem {
    type Error: From<err::Error>;
    type DirEntry: DirEntryTrait;
    type ReadDir: Iterator<Item = Result<Self::DirEntry, Self::Error>>;
    type Metadata: MetadataTrait;

    fn is_dir(&self, path: &PathBuf) -> bool {
        self.metadata(path).map(|a| a.is_dir()).unwrap_or(false)
    }

    fn read_dir(&self, path: &PathBuf) -> Result<Self::ReadDir, Self::Error>;
    fn metadata(&self, path: &PathBuf) -> Result<Self::Metadata, Self::Error>;
}

pub struct LinuxFS {}

impl FileSystem for LinuxFS {
    type Error = err::Error;
    type DirEntry = DirEntryWrapper;
    type ReadDir = ReadDirWrapper;
    type Metadata = MetadataWrapper;

    fn read_dir(&self, path: &PathBuf) -> Result<Self::ReadDir, Self::Error> {
        Ok(ReadDirWrapper(read_dir(path)?))
    }

    fn metadata(&self, path: &PathBuf) -> Result<Self::Metadata, Self::Error> {
        Ok(MetadataWrapper(fs::metadata(path)?))
    }
}

pub struct WalkBuilder<T: FileSystem> {
    fs: T,
    path: Option<PathBuf>,
    filters: Vec<Box<dyn PathFilter>>,
}

impl<'a, T: FileSystem> WalkBuilder<T> {
    pub fn new(file_system: T) -> WalkBuilder<T> {
        WalkBuilder {
            fs: file_system,
            path: None,
            filters: vec![],
        }
    }

    pub fn build(self) -> Result<Walk<T>, err::Error> {
        let path = self.path.ok_or(err::Error::NoPathDefined)?;
        Walk::new(self.fs, path, self.filters)
    }

    pub fn add_filter(mut self, path_filter: Box<dyn PathFilter>) -> Self {
        self.filters.push(path_filter);
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

impl<'a> Default for WalkBuilder<LinuxFS> {
    fn default() -> WalkBuilder<LinuxFS> {
        WalkBuilder {
            fs: LinuxFS {},
            path: None,
            filters: vec![],
        }
    }
}

pub struct Walk<T: FileSystem> {
    path_stack: Box<Vec<(usize, PathBuf)>>,
    dir_entries_stack: Box<Vec<PathBuf>>,
    fs: T,
    max_depth: Option<usize>,
    filters: Vec<Box<dyn PathFilter>>,
}

impl<T: FileSystem> fmt::Debug for Walk<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Walk")
            .field("path_stack", &self.path_stack)
            .field("dir_entries_stack", &self.dir_entries_stack)
            .field("max_depth", &self.max_depth)
            .finish()
    }
}

impl<T: FileSystem> Walk<T> {
    pub fn new(
        fs: T,
        path: PathBuf,
        filters: Vec<Box<dyn PathFilter>>,
    ) -> Result<Walk<T>, err::Error> {
        let mut filter_mut = filters;
        let path_str = path.to_str().ok_or(err::Error::PathFormatError)?;
        if path_str.contains('*') || path_str.contains('?') {
            let glob_filter = GlobFilter::new(path_str)?;
            filter_mut.push(Box::new(glob_filter));
        }

        let root_path = path_root(path_str);
        let max_depth;

        let pattern_root = pattern_root(path_str);

        let filter = StartFilter::new(pattern_root);
        filter_mut.push(Box::new(filter));
        if path_str.contains("**") {
            max_depth = None;
        } else {
            // root_path has been built upon path. We can unwrap
            let rest_path = path.strip_prefix(&root_path).unwrap();
            max_depth = Some(rest_path.components().count())
        }

        let mut path_stack = vec![];
        let mut dir_entries = vec![];
        if fs.is_dir(&root_path) {
            path_stack.push((0, root_path))
        } else {
            dir_entries.push(root_path)
        }
        Ok(Walk {
            path_stack: Box::new(path_stack),
            dir_entries_stack: Box::new(dir_entries),
            fs: fs,
            max_depth: max_depth,
            filters: filter_mut,
        })
    }

    fn next_file_entry(&mut self) -> Option<Result<PathBuf, T::Error>> {
        while let Some(entry) = self.dir_entries_stack.pop() {
            let pass = self
                .filters
                .iter()
                .all(|f| entry.to_str().map_or(false, |s| f.is_match(s)));

            if pass {
                return Some(Ok(entry));
            }
        }
        return None;
    }

    fn next_dir_entry(&mut self) -> Option<Result<(usize, PathBuf), T::Error>> {
        while let Some((depth, next_path)) = self.path_stack.pop() {
            if let Some(md) = self.max_depth {
                if depth > md {
                    continue;
                }
            }
            return Some(Ok((depth, next_path)));
        }

        return None;
    }
}

fn path_root(path: &str) -> PathBuf {
    let mut s = PathBuf::new();
    let mut slice = String::new();
    for token in path.chars() {
        match token {
            '*' | '?' | '[' => break,
            '/' => {
                slice.push(token);
                s.push(slice);
                slice = String::new();
            }
            _ => slice.push(token),
        }
    }
    return s;
}

fn pattern_root(path: &str) -> String {
    let mut s = String::new();
    for token in path.chars() {
        match token {
            '*' | '?' | '[' => break,
            _ => s.push(token),
        }
    }
    return s;
}

impl<T: FileSystem> Iterator for Walk<T> {
    type Item = Result<PathBuf, T::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.dir_entries_stack.len() == 0 && self.path_stack.len() == 0 {
                return None;
            }
            let entry = self.next_file_entry();
            if entry.is_some() {
                return entry;
            }

            let dir_entry = self.next_dir_entry();

            match dir_entry {
                Some(Ok((depth, dir_entry))) => {
                    for entry in try_opt_res!(self.fs.read_dir(&dir_entry)) {
                        let entry = try_opt_res!(entry);
                        let path = entry.path();

                        if self.fs.is_dir(&path)
                            && self.max_depth.map(|md| md > depth + 1).unwrap_or(false)
                        {
                            self.path_stack.push((depth + 1, path))
                        } else {
                            self.dir_entries_stack.push(path)
                        }
                    }
                }
                Some(Err(err)) => return Some(Err(err)),
                None => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use walk::*;
    #[test]
    fn can_read_dir() {
        let fs = LinuxFS {};
        let path = PathBuf::from("/opt/libhdfs3");
        for p in Walk::new(fs, path, vec![]).unwrap() {
            let p = p.unwrap();
            //println!("{:?}", p);
        }
    }

    #[test]

    fn can_read_dir_with_filter() {
        let path = PathBuf::from("/opt/libhdfs3");
        let filter = TestFilter {};
        for p in WalkBuilder::default()
            .with_path(path)
            .add_filter(Box::new(filter))
            .build()
            .unwrap()
        {
            let p = p.unwrap();
            //println!("{:?}", p);
        }
    }

    #[test]
    fn can_read_dir_with_glob() {
        let fs = LinuxFS {};
        let path = PathBuf::from("/opt/lib*/**");
        let walk = WalkBuilder::default().with_path(path).build().unwrap();
        println!("{:?}", walk);
        let mut count = 0;
        for p in walk {
            println!("{:?}", p);
            count += 1;
        }
        println!("count: {}", count);
    }
}
