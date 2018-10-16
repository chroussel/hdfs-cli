#![allow(unused_imports)]
use conf;
use err::HdfsErr;
use glob::Pattern;
use libc::{
    c_char, c_int, c_short, c_uchar, c_void, int16_t, int32_t, int64_t, size_t, time_t, uint16_t,
};
use native;
use nix::fcntl::OFlag;
use std::cmp;
use std::fmt;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::path::{self, Component, Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::slice;
use util::*;

const HOST_STRING: &'static str = "host";
const HOST_PORT: &'static str = "port";

pub struct HDFileSystem {
    raw: *const native::hdfsFS,
}

pub fn get_hdfs(
    config_path: &Path,
    host: Option<&str>,
    effective_user: Option<&str>,
) -> Result<HDFileSystem, HdfsErr> {
    let config = conf::Config::new(config_path)?;

    let namenode = match host.or(config.get_string(HOST_STRING)) {
        Some(name) => Ok(name),
        None => Err(HdfsErr::MissingConfig(String::from("host config missing"))),
    }?;

    let builder = unsafe { native::hdfsNewBuilder() };
    if builder.is_null() {
        return Err(HdfsErr::ErrorCreatingBuilder);
    }

    info!("Set namenode to {}", namenode);

    unsafe { native::hdfsBuilderSetForceNewInstance(builder) };

    unsafe { native::hdfsBuilderSetNameNode(builder, str_to_chars(namenode)) };

    if let Some(port) = config.get::<u16>(HOST_PORT) {
        info!("Set port to {}", port);
        unsafe { native::hdfsBuilderSetNameNodePort(builder, port) };
    }

    for (key, val) in config.get_all_key_values() {
        debug!("Setting {} to {}", key, val);
        let res = unsafe {
            native::hdfsBuilderConfSetStr(
                builder,
                str_to_chars(key.as_str()),
                str_to_chars(val.as_str()),
            )
        };
        if res != 0 {
            let reason = unsafe { chars_to_str(native::hdfsGetLastError()) };
            warn!("Conf cannot be set {} -> {} (reason: {})", key, val, reason)
        }
    }

    info!("Connecting to namenode {}", &namenode);
    let hdfs = unsafe {
        native::hdfsBuilderConnect(
            builder,
            effective_user
                .map(|u| str_to_chars(u))
                .unwrap_or(std::ptr::null()),
        )
    };
    unsafe { native::hdfsFreeBuilder(builder) };

    if hdfs.is_null() {
        info!("There was a connection error");
        Err(HdfsErr::get_last_error())
    } else {
        Ok(HDFileSystem { raw: hdfs })
    }
}

enum ObjectKind {
    Unknown,
    File,
    Directory,
}

impl ObjectKind {
    fn fromtObjectKind(tObjectKind: native::tObjectKind) -> ObjectKind {
        match tObjectKind {
            native::tObjectKind::kObjectKindFile => ObjectKind::File,
            native::tObjectKind::kObjectKindDirectory => ObjectKind::Directory,
            _ => ObjectKind::Unknown,
        }
    }
}

pub struct FileInfo {
    pub name: String,
    pub kind: ObjectKind,
}

pub struct Listing {
    pub path: String,
    pub kind: ObjectKind,
    pub files: Vec<FileInfo>,
}

impl HDFileSystem {
    pub fn globPath(&self, pattern: &str) -> Result<Vec<Listing>, HdfsErr> {
        let _pattern = Pattern::new(pattern)?;

        let components = Path::new(pattern).components().peekable();

        loop {
            match components.peek() {
                Some(&Component::RootDir) => {
                    components.next();
                }
                _ => {
                    break;
                }
            }
        }

        let rest = components.map(|s| s.as_os_str()).collect::<PathBuf>();
        let normalized_pattern = Path::new(pattern).iter().collect::<PathBuf>();
        let root_len = normalized_pattern.to_str().unwrap().len() - rest.to_str().unwrap().len();
        let root = if root_len > 0 {
            Some(Path::new(&pattern[..root_len]))
        } else {
            None
        };

        let scope = root
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut dir_patterns = Vec::new();
        let components =
            pattern[cmp::min(root_len, pattern.len())..].split_terminator(path::is_separator);

        for component in components {
            let compiled = try!(Pattern::new(component));
            dir_patterns.push(compiled);
        }

        if root_len == pattern.len() {
            dir_patterns.push(Pattern {
                original: "".to_string(),
                tokens: Vec::new(),
                is_recursive: false,
            });
        }

        let require_dir = pattern.chars().next_back().map(path::is_separator) == Some(true);
        let todo = Vec::new();

        Ok(Paths {
            dir_patterns: dir_patterns,
            require_dir: require_dir,
            options: options.clone(),
            todo: todo,
            scope: Some(scope),
        })
    }

    fn globPathR(&self, splittedPath: Vec<&str>) -> Result<Vec<Listing>, HdfsErr> {
        Ok(vec![])
    }

    fn hdfsfileToFileInfo(file: &native::hdfsFileInfo) -> FileInfo {
        FileInfo {
            name: chars_to_str(file.mName).to_owned(),
            kind: ObjectKind::fromtObjectKind(file.mKind),
        }
    }

    pub fn listDirectory(&self, path: &str) -> Result<Listing, HdfsErr> {
        let mut count: c_int = 0;

        let array_ptr =
            unsafe { native::hdfsListDirectory(self.raw, str_to_chars(path), &mut count) };

        if array_ptr.is_null() {
            return Err(HdfsErr::Unknown);
        }

        let list = unsafe {
            slice::from_raw_parts(array_ptr as *const native::hdfsFileInfo, count as usize)
        };

        let vec: Vec<FileInfo> = list
            .iter()
            .map(|file| HDFileSystem::hdfsfileToFileInfo(file))
            .collect();

        unsafe { native::hdfsFreeFileInfo(array_ptr, count) };
        return Ok(Listing {
            path: path.to_owned(),
            kind: ObjectKind::Directory,
            files: vec,
        });
    }

    /**
     * hdfsOpenFile - Open a hdfs file in given mode.
     * @param fs The configured filesystem handle.
     * @param path The full path to the file.
     * @param flags - an | of bits/fcntl.h file flags - supported flags are O_RDONLY, O_WRONLY (meaning create or overwrite i.e., implies O_TRUNCAT),
     * O_WRONLY|O_APPEND and O_SYNC. Other flags are generally ignored other than (O_RDWR || (O_EXCL & O_CREAT)) which return NULL and set errno equal ENOTSUP.
     * @param bufferSize Size of buffer for read/write - pass 0 if you want
     * to use the default configured values.
     * @param replication Block replication - pass 0 if you want to use
     * the default configured values.
     * @param blocksize Size of block - pass 0 if you want to use the
     * default configured values.
     * @return Returns the handle to the open file or NULL on error.
     */

    pub fn open_with_options(&self, path: &str, options: &OpenOptions) -> Result<File, HdfsErr> {
        let mut flag = OFlag::empty();

        flag.set(OFlag::O_CREAT, options.create);
        flag.set(OFlag::O_WRONLY, options.write);
        flag.set(OFlag::O_APPEND, options.append);
        flag.set(OFlag::O_RDONLY, options.read);

        let f = unsafe { native::hdfsOpenFile(self.raw, str_to_chars(path), flag.bits(), 0, 0, 0) };

        if f.is_null() {
            Err(HdfsErr::get_last_error())
        } else {
            Ok(File {
                fs: self.raw,
                raw: f,
            })
        }
    }

    pub fn exists(&self, path: &str) -> Result<bool, HdfsErr> {
        let res = unsafe { native::hdfsExists(self.raw, str_to_chars(path)) };

        if res == 0 {
            Ok(true)
        } else {
            let lastError = HdfsErr::get_last_hdfs_error();
            if let HdfsErr::NoError() = lastError {
                return Ok(false);
            } else {
                return Err(lastError);
            }
        }
    }
}

impl Drop for HDFileSystem {
    fn drop(&mut self) {
        let res = unsafe { native::hdfsDisconnect(self.raw) };
        if res != 0 {
            let lastError = HdfsErr::get_last_hdfs_error();
            warn!("{:?}", lastError)
        }
    }
}

pub struct File {
    fs: *const native::hdfsFS,
    raw: *const native::hdfsFile,
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let read_byte = unsafe {
            native::hdfsRead(
                self.fs,
                self.raw,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as i32,
            )
        };
        if read_byte >= 0 {
            Ok(read_byte as usize)
        } else {
            Err(Error::last_os_error())
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let b = buf;
        let write_byte = unsafe {
            native::hdfsWrite(
                self.fs,
                self.raw,
                b.as_ptr() as *const c_void,
                b.len() as i32,
            )
        };

        if write_byte >= 0 {
            Ok(write_byte as usize)
        } else {
            Err(Error::last_os_error())
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        let flush_byte = unsafe { native::hdfsFlush(self.fs, self.raw) };
        if flush_byte >= 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { native::hdfsCloseFile(self.fs, self.raw) };
    }
}

pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    create: bool,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to false, except for `read`.
    pub fn new() -> Self {
        OpenOptions {
            read: true,
            write: false,
            append: false,
            create: false,
        }
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous content. Note that setting
    /// `.write(true).append(true)` has the same effect as setting only
    /// `.append(true)`.
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.append = append;
        if append {
            self.write = true;
        }
        self
    }

    /// Sets the option for creating a new file.
    ///
    /// This option indicates whether a new file will be created if the file
    /// does not yet already exist.
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.create = create;
        if create {
            self.write = true;
        }
        self
    }

    pub fn open<P: AsRef<Path>>(&self, fs: &HDFileSystem, path: P) -> Result<File, HdfsErr> {
        let path = path.as_ref();
        let pathStr = path.to_str().ok_or(HdfsErr::PathConversionError(
            path.to_string_lossy().into_owned(),
        ))?;
        fs.open_with_options(pathStr, self)
    }
}
