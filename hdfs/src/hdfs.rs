use err::HdfsErr;
use libc::{
    c_char, c_int, c_short, c_uchar, c_void, int16_t, int32_t, int64_t, size_t, time_t, uint16_t,
};
use conf;
use native;
use std::fmt;
use std::mem;
use std::ptr;
use std::rc::Rc;
use std::path::Path;
use util::*;

pub struct HDFileSystem {
    raw: *const native::hdfsFS,
}

pub fn get_hdfs(config_path: &Path) -> Result<HDFileSystem, HdfsErr> {
    let hdfs = unsafe {
        let config = conf::Config::new(config_path);

        let builder = native::hdfsNewBuilder();
        native::hdfsBuilderSetNameNode(builder, str_to_chars(namenode));
        native::hdfsBuilderSetNameNodePort(bui)


        info!("Connecting to namenode {}", &namenode);
        native::hdfsBuilderConnect(builder)
    };

    if hdfs.is_null() {
        let reason = unsafe { chars_to_str(native::hdfsGetLastError()) };
        let res = format!("Cannot connect to namenode {}: {}", &namenode, &reason);
        return Err(HdfsErr::CannotConnectToNameNode(String::from(namenode)));
    } else {
        return Ok(HDFileSystem { raw: hdfs });
    }
}

enum ObjectKind {
    File,
    Directory,
}

pub struct FileInfo {
    pub name: String,
}

impl HDFileSystem {
    pub fn ls(&self, path: &str) -> Result<Vec<FileInfo>, String> {
        let mut count: c_int = 0;

        let array_ptr =
            unsafe { native::hdfsListDirectory(self.raw, str_to_chars(path), &mut count) };

        if array_ptr.is_null() {
            return Err(String::from("Error"));
        }

        let mut list = Vec::new();
        for idx in 0..count {
            let current = unsafe { array_ptr.offset(idx as isize) };
            list.push(FileInfo {
                name: String::from(chars_to_str(unsafe { &*current }.mName)),
            })
        }

        return Ok(list);
    }
}
