use conf;
use err::HdfsErr;
use libc::{
    c_char, c_int, c_short, c_uchar, c_void, int16_t, int32_t, int64_t, size_t, time_t, uint16_t,
};
use native;
use std::fmt;
use std::mem;
use std::path::Path;
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
    host: Option<&String>,
    effective_user: Option<&String>,
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
    unsafe { native::hdfsBuilderSetNameNode(builder, str_to_chars(namenode.as_str())) };

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
        let reason = unsafe { chars_to_str(native::hdfsGetLastError()) };
        return Err(HdfsErr::CannotConnectToNameNode((
            namenode.to_owned(),
            reason.to_owned(),
        )));
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
    pub fn ls(&self, path: &str) -> Result<Vec<FileInfo>, HdfsErr> {
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
            .map(|file| FileInfo {
                name: chars_to_str(file.mName).to_owned(),
            })
            .collect();
        return Ok(vec);
    }
}
