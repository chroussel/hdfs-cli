//! libhdfs FFI Binding APIs
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use libc::{
    c_char, c_int, c_short, c_uchar, c_void, int16_t, int32_t, int64_t, size_t, time_t, uint16_t,
};
use std::ptr;

type tTime = time_t;
type tOffset = int64_t;
type tPort = uint16_t;

#[repr(C)]
pub enum tObjectKind {
    kObjectKindFile = 0x46,      // 'F'
    kObjectKindDirectory = 0x44, // 'D'
}

#[repr(C)]
pub struct hdfsFileInfo {
    /// file or directory
    pub mKind: tObjectKind,
    /// the name of the file
    pub mName: *const c_char,
    /// the last modification time for the file in seconds
    pub mLastMod: tTime,
    /// the size of the file in bytes
    pub mSize: tOffset,
    /// the count of replicas
    pub mReplication: c_short,
    /// the block size for the file
    pub mBlockSize: tOffset,
    /// the owner of the file
    pub mOwner: *const c_char,
    /// the group associated with the file
    pub mGroup: *const c_char,
    /// the permissions associated with the file
    pub mPermissions: c_short,
    /// the last access time for the file in seconds
    pub mLastAccess: tTime,
}

pub enum hdfsBuilder {}
pub enum hdfsFS {}

#[link(name = "hdfs3")]
extern "C" {
    pub fn hdfsNewBuilder() -> *const hdfsBuilder;

    pub fn hdfsBuilderConnect(bld: *const hdfsBuilder) -> *const hdfsFS;
    pub fn hdfsBuilderSetNameNode(bld: *const hdfsBuilder, namenode: *const c_char);
    pub fn hdfsBuilderSetNameNodePort(bld: *const hdfsBuilder, port: tPort);
    pub fn hdfsGetLastError() -> *const c_char;
    pub fn hdfsListDirectory(
        fs: *const hdfsFS,
        path: *const c_char,
        numEntries: *mut c_int,
    ) -> *const hdfsFileInfo;
}
