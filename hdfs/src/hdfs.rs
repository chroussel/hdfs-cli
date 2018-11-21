#![allow(unused_imports)]
use config;
use err::Error;
use libc::{
    c_char, c_int, c_short, c_uchar, c_void, int16_t, int32_t, int64_t, size_t, time_t, uint16_t,
};
use native;
use nix::fcntl::OFlag;
use std::cmp;
use std::fmt;
use std::io;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::path::{self, Component, Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::slice;
use util::*;

const HOST_STRING: &str = "host";
const HOST_PORT: &str = "port";
const GATEWAYS: &str = "dfs.nameservices";

pub struct HDFileSystem {
    raw: *const native::hdfsFS,
}

pub fn list_gateway<P: AsRef<Path>>(config_path: P) -> Result<Vec<String>, Error> {
    let config = config::Config::new(config_path.as_ref())?;

    if let Some(gateways) = config.get_string(GATEWAYS) {
        Ok(gateways.split(',').map(|s| s.to_owned()).collect())
    } else {
        return Ok(vec![]);
    }
}

pub fn get_hdfs<P: AsRef<Path>>(
    config_path: P,
    host: Option<&str>,
    effective_user: Option<&str>,
) -> Result<HDFileSystem, Error> {
    let config = config::Config::new(config_path.as_ref())?;

    let namenode = match host.or_else(|| config.get_string(HOST_STRING)) {
        Some(name) => Ok(name),
        None => Err(Error::MissingConfig(String::from("host config missing"))),
    }?;

    let builder = unsafe { native::hdfsNewBuilder() };
    if builder.is_null() {
        return Err(Error::ErrorCreatingBuilder);
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
        Err(Error::get_last_error())
    } else {
        Ok(HDFileSystem { raw: hdfs })
    }
}

#[derive(PartialEq)]
pub enum ObjectKind {
    Unknown,
    File,
    Directory,
}

impl ObjectKind {
    fn from_t_object_kind(kind: &native::tObjectKind) -> ObjectKind {
        match kind {
            native::tObjectKind::kObjectKindFile => ObjectKind::File,
            native::tObjectKind::kObjectKindDirectory => ObjectKind::Directory,
        }
    }
}

pub struct DirEntry {
    path: PathBuf,
    kind: ObjectKind,
}

impl DirEntry {
    pub fn is_dir(&self) -> bool {
        self.kind == ObjectKind::Directory
    }

    pub fn is_file(&self) -> bool {
        self.kind == ObjectKind::File
    }

    pub fn path(&self) -> PathBuf {
        self.path.to_owned()
    }
}

pub struct ReadDir {
    pub path: PathBuf,
    pub kind: ObjectKind,
    pub files: Vec<DirEntry>,
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.files.pop()
    }
}

impl HDFileSystem {
    fn hdfsfile_to_file_info(file: &native::hdfsFileInfo) -> DirEntry {
        DirEntry {
            path: PathBuf::from(chars_to_str(file.mName)),
            kind: ObjectKind::from_t_object_kind(&file.mKind),
        }
    }

    pub fn path_info(&self, path: &PathBuf) -> Result<DirEntry, Error> {
        let path_str = path.to_str();
        if path_str.is_none() {
            return Err(Error::InvalidPath(path.to_owned()));
        }

        let path_str = path_str.unwrap();

        let array_ptr = unsafe { native::hdfsGetPathInfo(self.raw, str_to_chars(path_str)) };

        let mut vec = HDFileSystem::convert_file_info_to_vec(array_ptr, 1)?;
        let file = vec.pop();

        if file.is_none() {
            return Err(Error::Unknown);
        }

        let file = file.unwrap();
        Ok(file)
    }

    pub fn list_directory(&self, path: &PathBuf) -> Result<ReadDir, Error> {
        let mut count: c_int = 0;
        let path_str = path.to_str();
        if path_str.is_none() {
            return Err(Error::InvalidPath(path.to_owned()));
        }

        let path_str = path_str.unwrap();

        let array_ptr =
            unsafe { native::hdfsListDirectory(self.raw, str_to_chars(path_str), &mut count) };

        let vec = HDFileSystem::convert_file_info_to_vec(array_ptr, count)?;

        Ok(ReadDir {
            path: path.to_owned(),
            kind: ObjectKind::Directory,
            files: vec,
        })
    }

    fn convert_file_info_to_vec(
        array_ptr: *const native::hdfsFileInfo,
        count: i32,
    ) -> Result<Vec<DirEntry>, Error> {
        if array_ptr.is_null() {
            return Err(Error::Unknown);
        }

        let list = unsafe {
            slice::from_raw_parts(array_ptr as *const native::hdfsFileInfo, count as usize)
        };

        let vec: Vec<DirEntry> = list
            .iter()
            .map(|file| HDFileSystem::hdfsfile_to_file_info(file))
            .collect();

        unsafe { native::hdfsFreeFileInfo(array_ptr, count) };
        Ok(vec)
    }

    pub fn open_with_options<P: AsRef<Path>>(
        &self,
        path: P,
        options: &OpenOptions,
    ) -> Result<File, Error> {
        let mut flag = OFlag::empty();

        flag.set(OFlag::O_CREAT, options.create);
        flag.set(OFlag::O_WRONLY, options.write);
        flag.set(OFlag::O_APPEND, options.append);
        flag.set(OFlag::O_RDONLY, options.read);
        let path = path.as_ref();
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::PathConversionError(path.to_string_lossy().into_owned()))?;
        let f =
            unsafe { native::hdfsOpenFile(self.raw, str_to_chars(path_str), flag.bits(), 0, 0, 0) };

        if f.is_null() {
            Err(Error::get_last_error())
        } else {
            Ok(File {
                fs: self.raw,
                raw: f,
            })
        }
    }

    pub fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Error> {
        let path_str = path.as_ref().to_str();
        if path_str.is_none() {
            return Err(Error::InvalidPath(path.as_ref().to_owned()));
        }
        let path_str = path_str.unwrap();
        let res = unsafe { native::hdfsExists(self.raw, str_to_chars(path_str)) };

        if res == 0 {
            Ok(true)
        } else {
            let last_error = Error::get_last_hdfs_error();
            if let Error::NoError() = last_error {
                return Ok(false);
            } else {
                return Err(last_error);
            }
        }
    }
}

impl Drop for HDFileSystem {
    fn drop(&mut self) {
        let res = unsafe { native::hdfsDisconnect(self.raw) };
        if res != 0 {
            let last_error = Error::get_last_hdfs_error();
            warn!("{:?}", last_error)
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
            Err(io::Error::last_os_error())
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
            Err(io::Error::last_os_error())
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        let flush_byte = unsafe { native::hdfsFlush(self.fs, self.raw) };
        if flush_byte >= 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
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

impl Default for OpenOptions {
    fn default() -> OpenOptions {
        OpenOptions {
            read: true,
            write: false,
            append: false,
            create: false,
        }
    }
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

    pub fn open<P: AsRef<Path>>(&self, fs: &HDFileSystem, path: P) -> Result<File, Error> {
        fs.open_with_options(path, self)
    }
}
