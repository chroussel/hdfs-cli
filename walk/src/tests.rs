use super::err;
use super::walk::*;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::Component;
use std::path::PathBuf;

#[derive(Debug)]
enum Error {
    NotFound(PathBuf),
    Io(io::Error),
    Inner(err::Error),
}

impl From<err::Error> for Error {
    fn from(e: err::Error) -> Error {
        Error::Inner(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

#[derive(Default)]
struct MockFS {
    root: Node,
}

impl MockFS {
    fn new(root: Node) -> MockFS {
        MockFS { root }
    }

    fn find_node(&self, path: &PathBuf) -> Result<&Node, Error> {
        let mut current_node = &self.root;

        if current_node.path() == path.as_os_str() {
            return Ok(current_node);
        }

        for c in path.components() {
            match c {
                Component::RootDir => continue,
                Component::Normal(value) => {
                    current_node = match current_node {
                        Node::Dir(_, entries) => {
                            println!("{:?}", entries);
                            println!("{:?}", value.to_str().unwrap());
                            entries
                                .get(value)
                                .ok_or_else(|| Error::NotFound(path.clone()))
                        }
                        _ => return Err(Error::NotFound(path.clone())),
                    }?;
                }
                _ => return Err(Error::NotFound(path.clone())),
            }
        }

        Ok(current_node)
    }
}

enum NodeBuilder {
    File(PathBuf),
    Dir(PathBuf, HashMap<OsString, NodeBuilder>),
}

impl NodeBuilder {
    fn path(&self) -> &PathBuf {
        match self {
            NodeBuilder::Dir(path, _) => path,
            NodeBuilder::File(path) => path,
        }
    }

    fn build(self, parent_path: &PathBuf) -> Node {
        match self {
            NodeBuilder::File(name) => Node::File(parent_path.clone().join(name)),
            NodeBuilder::Dir(name, children) => {
                let path = parent_path.clone().join(name);
                let dir_path = path.clone();
                let map = children
                    .into_iter()
                    .map(move |(k, v)| (k, v.build(&path)))
                    .collect();
                Node::Dir(dir_path, map)
            }
        }
    }
}

#[derive(Debug)]
enum Node {
    File(PathBuf),
    Dir(PathBuf, HashMap<OsString, Node>),
}

impl Default for Node {
    fn default() -> Node {
        Node::Dir(PathBuf::from("/"), HashMap::default())
    }
}

impl Node {
    fn path(&self) -> &PathBuf {
        match self {
            Node::Dir(path, _) => path,
            Node::File(path) => path,
        }
    }

    fn to_read_dir_node(&self) -> ReadDirNode {
        let entries = match self {
            Node::File(_) => vec![self.to_dir_entry_node()],
            Node::Dir(_, entries) => entries.iter().map(|(_, v)| v.to_dir_entry_node()).collect(),
        };
        ReadDirNode { entries }
    }

    fn to_dir_entry_node(&self) -> DirEntryNode {
        let (path, is_dir) = match self {
            Node::File(path) => (path, false),
            Node::Dir(path, _) => (path, true),
        };
        DirEntryNode {
            path: path.clone(),
            is_dir,
        }
    }
}

struct DirEntryNode {
    path: PathBuf,
    is_dir: bool,
}

impl DirEntryTrait for DirEntryNode {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn is_dir(&self) -> bool {
        self.is_dir
    }
}
impl MetadataTrait for DirEntryNode {
    fn is_dir(&self) -> bool {
        self.is_dir
    }
}

struct ReadDirNode {
    entries: Vec<DirEntryNode>,
}

impl Iterator for ReadDirNode {
    type Item = Result<DirEntryNode, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop().map(Ok)
    }
}

impl FileSystem for MockFS {
    type Error = Error;
    type DirEntry = DirEntryNode;
    type ReadDir = ReadDirNode;
    type Metadata = DirEntryNode;

    fn exists(&self, path: &PathBuf) -> bool {
        self.find_node(path).is_ok()
    }

    fn read_dir(&self, path: &PathBuf) -> Result<Self::ReadDir, Self::Error> {
        self.find_node(path).map(|n| n.to_read_dir_node())
    }

    fn metadata(&self, path: &PathBuf) -> Result<Self::Metadata, Self::Error> {
        self.find_node(path).map(|n| n.to_dir_entry_node())
    }
}

fn td(name: &str, children: Vec<NodeBuilder>) -> NodeBuilder {
    let hashmap: HashMap<OsString, NodeBuilder> = children
        .into_iter()
        .map(|c| (c.path().clone().into_os_string(), c))
        .collect();
    NodeBuilder::Dir(PathBuf::from(name.to_owned()), hashmap)
}

fn tf(name: &str) -> NodeBuilder {
    NodeBuilder::File(PathBuf::from(name))
}

fn path_list(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

#[test]
fn walk_1() {
    let _fs = MockFS::default();
}

#[test]
fn walk_2() {
    let tree = td("/", vec![td("var", vec![]), tf("file")]);

    let _fs = MockFS::new(tree.build(&PathBuf::from("")));
}

#[test]
fn walk_3() {
    let tree = td("/", vec![td("var", vec![]), tf("file")]);

    let fs = MockFS::new(tree.build(&PathBuf::from("")));

    let entries: Result<Vec<_>, _> = fs.read_dir(&PathBuf::from("/")).unwrap().collect();
    let mut paths: Vec<_> = entries.unwrap().into_iter().map(|de| de.path).collect();
    paths.sort();
    assert_eq!(paths, path_list(&["/file", "/var"]));
}

#[test]
fn walk_4() {
    let tree = td(
        "/",
        vec![td("var", vec![tf("file1"), tf("file2")]), tf("file")],
    );
    let fs = MockFS::new(tree.build(&PathBuf::from("/")));

    let entries: Result<Vec<_>, _> = fs.read_dir(&PathBuf::from("/var")).unwrap().collect();
    let mut paths: Vec<_> = entries.unwrap().into_iter().map(|de| de.path).collect();
    paths.sort();
    assert_eq!(paths, path_list(&["/var/file1", "/var/file2"]));
}

fn enable_log() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");
    let _ = env_logger::Builder::from_env(env).try_init();
}

#[test]
fn test_list_root() {
    enable_log();
    let tree = td("/", vec![td("var", vec![]), tf("file")]);

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(list, path_list(&["/"]));
}

#[test]
fn test_list_directory_explicit() {
    enable_log();
    let tree = td(
        "/",
        vec![td("var", vec![tf("file1"), tf("file2")]), tf("file")],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/var/"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();

    assert_eq!(list, path_list(&["/var"]));
}

#[test]
fn test_list_directory_implicit() {
    enable_log();
    let tree = td(
        "/",
        vec![td("var", vec![tf("file1"), tf("file2")]), tf("file")],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/var"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(list, path_list(&["/var"]));
}

#[test]
fn test_list_directory_empty() {
    enable_log();
    let tree = td(
        "/",
        vec![td("var", vec![tf("file1"), tf("file2")]), tf("file")],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/va"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(0, list.len());
}

#[test]
fn test_list_directory_with_glob() {
    enable_log();
    let tree = td(
        "/",
        vec![td("var", vec![tf("file1"), tf("file2")]), tf("file")],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/va*"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(list, path_list(&["/var"]));
}

#[test]
fn test_list_directory_with_complex_glob() {
    enable_log();
    let tree = td(
        "/",
        vec![td("var", vec![tf("file1"), tf("file2")]), tf("file")],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/va*/fi*"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(list, path_list(&["/var/file1", "/var/file2"]));
}

#[test]
fn test_list_directory_with_question_mark() {
    enable_log();
    let tree = td(
        "/",
        vec![
            td(
                "var",
                vec![tf("file1"), tf("file2a"), tf("file3a"), tf("file2b")],
            ),
            tf("file"),
        ],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/var/file?a"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(list, path_list(&["/var/file2a", "/var/file3a"]));
}

#[test]
fn test_list_directory_with_multiple_glob() {
    enable_log();
    let tree = td(
        "/",
        vec![
            td("var", vec![tf("file1"), tf("file2")]),
            td("var2", vec![tf("file3"), tf("file4")]),
            td("bar3", vec![tf("file3"), tf("file4")]),
            td(
                "var3",
                vec![
                    td("var4", vec![tf("file5"), tf("file8")]),
                    td("var5", vec![tf("file2"), tf("file9")]),
                    td("var6", vec![tf("file7"), tf("file10"), td("var7", vec![])]),
                    tf("file11"),
                ],
            ),
            tf("file0"),
        ],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/*/*/*"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();
    assert_eq!(
        list,
        path_list(&[
            "/var3/var4/file5",
            "/var3/var4/file8",
            "/var3/var5/file2",
            "/var3/var5/file9",
            "/var3/var6/file10",
            "/var3/var6/file7",
            "/var3/var6/var7"
        ])
    );
}

#[test]
fn test_list_directory_with_globstar() {
    enable_log();
    let tree = td(
        "/",
        vec![
            td("var", vec![tf("file1"), tf("file2")]),
            td("var2", vec![tf("file3"), tf("file4")]),
            td(
                "var3",
                vec![
                    td("var4", vec![tf("file5"), tf("file8")]),
                    td("var5", vec![tf("file2"), tf("file9")]),
                    td("var6", vec![tf("file7"), tf("file10")]),
                    tf("file11"),
                ],
            ),
            tf("file0"),
        ],
    );

    let fs = MockFS::new(tree.build(&PathBuf::from("")));
    let walkbuilder = WalkBuilder::new(fs);
    let list: Result<Vec<_>, Error> = walkbuilder
        .with_path(PathBuf::from("/var*/**/file2"))
        .build()
        .unwrap()
        .collect();

    let mut list = list.unwrap();
    list.sort();

    assert_eq!(list, path_list(&["/var/file2", "/var3/var5/file2"]));
}
