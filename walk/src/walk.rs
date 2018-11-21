use err;
use filter::*;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

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
    fn is_dir(&self) -> bool;
}

pub trait FileSystem {
    type Error: From<err::Error>;
    type DirEntry: DirEntryTrait;
    type ReadDir: Iterator<Item = Result<Self::DirEntry, Self::Error>>;
    type Metadata: MetadataTrait;

    fn is_dir(&self, path: &PathBuf) -> bool {
        self.metadata(path).map(|a| a.is_dir()).unwrap_or(false)
    }

    fn exists(&self, path: &PathBuf) -> bool;
    fn read_dir(&self, path: &PathBuf) -> Result<Self::ReadDir, Self::Error>;
    fn metadata(&self, path: &PathBuf) -> Result<Self::Metadata, Self::Error>;
}

#[derive(Default)]
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

#[derive(Debug)]
enum Node {
    File(PathBuf),
    Dir(usize, PathBuf),
}

pub struct Walk<T: FileSystem> {
    path_stack: Vec<Node>,
    fs: T,
    max_depth: Option<usize>,
    filters: Vec<Box<dyn PathFilter>>,
}

impl<T: FileSystem> fmt::Debug for Walk<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Walk")
            .field("path_stack", &self.path_stack)
            .field("max_depth", &self.max_depth)
            .finish()
    }
}

impl<T: FileSystem> Walk<T> {
    pub fn new<P: AsRef<Path>>(
        fs: T,
        path: P,
        filters: Vec<Box<dyn PathFilter>>,
    ) -> Result<Walk<T>, err::Error> {
        let mut path_stack = vec![];
        let mut filter_mut = filters;
        let path: &Path = path.as_ref();
        let mut root_path = path.to_path_buf();
        let path_str = path.to_str().ok_or(err::Error::PathFormatError)?;

        if path_str.contains('*') || path_str.contains('?') {
            let globfilter = GlobFilter::new(path_str)?;
            filter_mut.push(Box::new(globfilter));
            root_path = path_root(path_str);
        }

        let max_depth = if path_str.contains("**") {
            None
        } else {
            Some(path.strip_prefix(&root_path).unwrap().components().count())
        };

        if fs.is_dir(&root_path) {
            path_stack.push(Node::Dir(0, root_path))
        } else if fs.exists(&root_path) {
            path_stack.push(Node::File(root_path))
        }

        debug!("max depth: {:?}", max_depth);
        Ok(Walk {
            path_stack,
            fs,
            max_depth,
            filters: filter_mut,
        })
    }

    fn is_valid(&self, path: &PathBuf) -> bool {
        let path_str = path.to_str();
        self.filters
            .iter()
            .all(|f| path_str.map_or(false, |s| f.is_match(s)))
    }

    fn resolve_next(&mut self) -> Option<Result<Node, T::Error>> {
        while let Some(node) = self.path_stack.pop() {
            debug!("resolve_next: {:?}", node);
            match node {
                Node::File(path) => {
                    if self.is_valid(&path) {
                        return Some(Ok(Node::File(path)));
                    }
                }
                Node::Dir(depth, path) => {
                    if self.max_depth.map(|md| depth < md).unwrap_or(true) {
                        try_opt_res!(self.fill_path_stack(&path, depth));
                    }

                    if self.is_valid(&path) {
                        return Some(Ok(Node::Dir(depth, path)));;
                    }
                }
            }
        }
        None
    }

    fn fill_path_stack(&mut self, path: &PathBuf, depth: usize) -> Result<(), T::Error> {
        for entry in self.fs.read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if entry.is_dir() {
                self.path_stack.push(Node::Dir(depth + 1, path))
            } else {
                self.path_stack.push(Node::File(path))
            }
        }
        Ok(())
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
    s
}

pub struct WalkItem {
    path: PathBuf,
    is_dir: bool,
}

impl WalkItem {
    fn new(path: PathBuf, is_dir: bool) -> WalkItem {
        WalkItem { path, is_dir }
    }

    pub fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

impl<T: FileSystem> Iterator for Walk<T> {
    type Item = Result<WalkItem, T::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.resolve_next() {
            match entry {
                Ok(Node::Dir(_, path)) => return Some(Ok(WalkItem::new(path, true))),
                Ok(Node::File(path)) => return Some(Ok(WalkItem::new(path, false))),
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}
