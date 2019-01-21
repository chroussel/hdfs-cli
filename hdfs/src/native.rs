//! libhdfs FFI Binding APIs
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![warn(dead_code)]

use libc::{
    c_char, c_int, c_short, c_uchar, c_void, int16_t, int32_t, int64_t, size_t, time_t, uint16_t,
};
use std::ptr;

type tSize = int32_t;
type tTime = time_t;
type tOffset = int64_t;
type tPort = uint16_t;

#[repr(C)]
pub enum tObjectKind {
    kObjectKindFile = 0x46,      // 'F'
    kObjectKindDirectory = 0x44, // 'D'
}

#[repr(C)]
pub struct hdfsEncryptionFileInfo {
    pub mSuite: c_int,
    pub mCryptoProtocolVersion: c_int,
    pub mKey: *const c_char,
    pub mKeyName: *const c_char,
    pub mIv: *const c_char,
    pub mEzKeyVersionName: *const c_char,
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

    pub mHdfsEncryptionFileInfo: *const hdfsEncryptionFileInfo,
}

pub enum hdfsFile {}
pub enum hdfsBuilder {}
pub enum hdfsFS {}

#[link(name = "hdfs3")]
extern "C" {
    pub fn hdfsNewBuilder() -> *mut hdfsBuilder;

    //, const char * effective_user=
    //pub fn hdfsBuilderConnect(bld: *const hdfsBuilder) -> *const hdfsFS;
    pub fn hdfsBuilderConnect(
        bld: *mut hdfsBuilder,
        effective_user: *const c_char,
    ) -> *const hdfsFS;
    pub fn hdfsBuilderSetForceNewInstance(bld: *mut hdfsBuilder);
    pub fn hdfsBuilderSetNameNode(bld: *mut hdfsBuilder, namenode: *const c_char);
    pub fn hdfsBuilderSetNameNodePort(bld: *mut hdfsBuilder, port: tPort);
    pub fn hdfsFreeBuilder(bld: *mut hdfsBuilder);
    pub fn hdfsBuilderConfSetStr(
        bld: *mut hdfsBuilder,
        key: *const c_char,
        val: *const c_char,
    ) -> c_int;

    pub fn hdfsDisconnect(fs: *const hdfsFS) -> c_int;
    pub fn hdfsGetLastError() -> *const c_char;
    pub fn hdfsListDirectory(
        fs: *const hdfsFS,
        path: *const c_char,
        numEntries: *mut c_int,
    ) -> *const hdfsFileInfo;

    pub fn hdfsFreeFileInfo(infos: *const hdfsFileInfo, numEntries: c_int);

    pub fn hdfsOpenFile(
        fs: *const hdfsFS,
        path: *const c_char,
        flags: c_int,
        bufferSize: c_int,
        replication: c_short,
        blocksize: tOffset,
    ) -> *const hdfsFile;

    pub fn hdfsCloseFile(fs: *const hdfsFS, file: *const hdfsFile) -> c_int;

    pub fn hdfsRead(
        fs: *const hdfsFS,
        file: *const hdfsFile,
        buffer: *mut c_void,
        length: tSize,
    ) -> tSize;

    pub fn hdfsWrite(
        fs: *const hdfsFS,
        file: *const hdfsFile,
        buffer: *const c_void,
        length: tSize,
    ) -> tSize;

    pub fn hdfsDelete(fs: *const hdfsFS, path: *const c_char) -> c_int;

    pub fn hdfsExists(fs: *const hdfsFS, path: *const c_char) -> c_int;

    pub fn hdfsFlush(fs: *const hdfsFS, file: *const hdfsFile) -> c_int;

    pub fn hdfsCopy(
        srcFS: *const hdfsFS,
        src: *const c_char,
        dstFS: *const hdfsFS,
        dst: *const c_char,
    );
    pub fn hdfsMove(
        srcFS: *const hdfsFS,
        src: *const c_char,
        dstFS: *const hdfsFS,
        dst: *const c_char,
    );
    pub fn hdfsRename(srcFS: *const hdfsFS, src: *const c_char, dst: *const c_char);
    pub fn hdfsCreateDirectory(fs: *const hdfsFS, path: *const c_char) -> c_int;
    pub fn hdfsGetPathInfo(fs: *const hdfsFS, path: *const c_char) -> *const hdfsFileInfo;
    pub fn hdfsGetWorkingDirectory(fs: *const hdfsFS, buffer: *const c_char, bufferSize: tSize);
}
