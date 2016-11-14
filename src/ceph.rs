// Copyright 2016 LambdaStack All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![cfg(target_os = "linux")]

use byteorder::{LittleEndian, WriteBytesExt};
use libc::{ENOENT, ERANGE, c_int, strerror, strlen, timeval};
use nom::{IResult, le_u32};
use rados::*;

use std::error::Error as StdError;
use std::ffi::{CStr, CString, IntoStringError, NulError};
use std::fmt;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Error;
use std::net::IpAddr;
use std::ptr;
use std::string::FromUtf8Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::{ParseError, Uuid};

const CEPH_OSD_TMAP_HDR: char = 'h';
const CEPH_OSD_TMAP_SET: char = 's';
const CEPH_OSD_TMAP_CREATE: char = 'c';
const CEPH_OSD_TMAP_RM: char = 'r';

/// Custom error handling for the library
#[derive(Debug)]
pub enum RadosError {
    FromUtf8Error(FromUtf8Error),
    NulError(NulError),
    Error(String),
    IoError(Error),
    IntoStringError(IntoStringError),
    ParseError(ParseError),
}

impl fmt::Display for RadosError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for RadosError {
    fn description(&self) -> &str {
        match *self {
            RadosError::FromUtf8Error(ref e) => e.description(),
            RadosError::NulError(ref e) => e.description(),
            RadosError::Error(ref e) => &e,
            RadosError::IoError(ref e) => e.description(),
            RadosError::IntoStringError(ref e) => e.description(),
            RadosError::ParseError(ref e) => e.description(),
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            RadosError::FromUtf8Error(ref e) => e.cause(),
            RadosError::NulError(ref e) => e.cause(),
            RadosError::Error(_) => None,
            RadosError::IoError(ref e) => e.cause(),
            RadosError::IntoStringError(ref e) => e.cause(),
            RadosError::ParseError(ref e) => e.cause(),
        }
    }
}

impl RadosError {
    /// Create a new RadosError with a String message
    pub fn new(err: String) -> RadosError {
        RadosError::Error(err)
    }

    /// Convert a RadosError into a String representation.
    pub fn to_string(&self) -> String {
        match *self {
            RadosError::FromUtf8Error(ref err) => err.utf8_error().to_string(),
            RadosError::NulError(ref err) => err.description().to_string(),
            RadosError::Error(ref err) => err.to_string(),
            RadosError::IoError(ref err) => err.description().to_string(),
            RadosError::IntoStringError(ref err) => err.description().to_string(),
            RadosError::ParseError(_) => "Uuid parse error".to_string(),
        }
    }
}

impl From<ParseError> for RadosError {
    fn from(err: ParseError) -> RadosError {
        RadosError::ParseError(err)
    }
}


impl From<NulError> for RadosError {
    fn from(err: NulError) -> RadosError {
        RadosError::NulError(err)
    }
}

impl From<FromUtf8Error> for RadosError {
    fn from(err: FromUtf8Error) -> RadosError {
        RadosError::FromUtf8Error(err)
    }
}
impl From<IntoStringError> for RadosError {
    fn from(err: IntoStringError) -> RadosError {
        RadosError::IntoStringError(err)
    }
}
impl From<Error> for RadosError {
    fn from(err: Error) -> RadosError {
        RadosError::IoError(err)
    }
}

fn get_error(n: c_int) -> Result<String, RadosError> {
    unsafe {
        let error_cstring = CString::from_raw(strerror(n));
        let message = try!(error_cstring.into_string());
        Ok(message)
    }
}

named!(parse_header<TmapOperation>,
    chain!(
        char!(CEPH_OSD_TMAP_HDR)~
        data_len: le_u32~
        data: take!(data_len),
        ||{
            let mut data_vec: Vec<u8> = Vec::new();
            data_vec.extend_from_slice(data);
            TmapOperation::Header{
                data: data_vec
            }
        }
    )
);

named!(parse_create<TmapOperation>,
    chain!(
        char!(CEPH_OSD_TMAP_CREATE)~
        key_name_len: le_u32~
        key_name: take_str!(key_name_len)~
        data_len: le_u32~
        data: take!(data_len),
        ||{
            let mut data_vec: Vec<u8> = Vec::new();
            data_vec.extend_from_slice(data);
            TmapOperation::Create{
                name: key_name.to_string(),
                data: data_vec
            }
        }
    )
);

named!(parse_set<TmapOperation>,
    chain!(
        char!(CEPH_OSD_TMAP_SET)~
        key_name_len: le_u32~
        key_name: take_str!(key_name_len)~
        data_len: le_u32~
        data: take!(data_len),
        ||{
            let mut data_vec: Vec<u8> = Vec::new();
            data_vec.extend_from_slice(data);
            TmapOperation::Set{
                key: key_name.to_string(),
                data: data_vec
            }
        }
    )
);

named!(parse_remove<TmapOperation>,
    chain!(
        char!(CEPH_OSD_TMAP_RM)~
        key_name_len: le_u32~
        key_name: take_str!(key_name_len),
        ||{
            TmapOperation::Remove{
                name: key_name.to_string()
            }
        }
    )
);

#[derive(Debug)]
pub enum TmapOperation {
    Header { data: Vec<u8>, },
    Set { key: String, data: Vec<u8>, },
    Create { name: String, data: Vec<u8>, },
    Remove { name: String, },
}

impl TmapOperation {
    fn serialize(&self) -> Result<Vec<u8>, RadosError> {
        let mut buffer: Vec<u8> = Vec::new();
        match self {
            &TmapOperation::Header { ref data } => {
                buffer.push(CEPH_OSD_TMAP_HDR as u8);
                try!(buffer.write_u32::<LittleEndian>(data.len() as u32));
                buffer.extend_from_slice(data);
            },
            &TmapOperation::Set { ref key, ref data } => {
                buffer.push(CEPH_OSD_TMAP_SET as u8);
                try!(buffer.write_u32::<LittleEndian>(key.len() as u32));
                buffer.extend(key.as_bytes());
                try!(buffer.write_u32::<LittleEndian>(data.len() as u32));
                buffer.extend_from_slice(data);
            },
            &TmapOperation::Create { ref name, ref data } => {
                buffer.push(CEPH_OSD_TMAP_CREATE as u8);
                try!(buffer.write_u32::<LittleEndian>(name.len() as u32));
                buffer.extend(name.as_bytes());
                try!(buffer.write_u32::<LittleEndian>(data.len() as u32));
                buffer.extend_from_slice(data);
            },
            &TmapOperation::Remove { ref name } => {
                buffer.push(CEPH_OSD_TMAP_RM as u8);
                try!(buffer.write_u32::<LittleEndian>(name.len() as u32));
                buffer.extend(name.as_bytes());
            },
        }
        Ok(buffer)
    }

    fn deserialize(input: &[u8]) -> IResult<&[u8], Vec<TmapOperation>> {
        many0!(input,
            alt!(
                complete!(parse_header)|
                complete!(parse_create) |
                complete!(parse_set) |
                complete!(parse_remove)
            )
        )
    }
}

/// Helper to iterate over pool objects
#[derive(Debug)]
pub struct Pool {
    pub ctx: rados_list_ctx_t,
}

impl Iterator for Pool {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        let mut entry_ptr: *mut *const ::libc::c_char = ptr::null_mut();
        let key_ptr: *const ::libc::c_char = ptr::null();
        let nspace_ptr: *const ::libc::c_char = ptr::null();

        unsafe {
            let ret_code = rados_nobjects_list_next(self.ctx,
                                                    &mut entry_ptr,
                                                    key_ptr as *mut *const i8,
                                                    nspace_ptr as *mut *const i8);
            if ret_code == -ENOENT {
                // We're done
                rados_nobjects_list_close(self.ctx);
                None
            } else if ret_code < 0 {
                // Unknown error
                None
            } else {
                let object_name = CStr::from_ptr(entry_ptr as *const ::libc::c_char);
                return Some(object_name.to_string_lossy().into_owned());
            }
        }
    }
}

/// A helper to create rados read operation
/// An object read operation stores a number of operations which can be executed atomically.
#[derive(Debug)]
pub struct ReadOperation {
    pub object_name: String,
    /// flags are set by calling LIBRADOS_OPERATION_NOFLAG | LIBRADOS_OPERATION_BALANCE_READS
    /// all the other flags are documented in rados.rs
    pub flags: u32,
    read_op_handle: rados_read_op_t,
}

impl Drop for ReadOperation {
    fn drop(&mut self) {
        unsafe {
            rados_release_read_op(self.read_op_handle);
        }
    }
}

/// A helper to create rados write operation
/// An object write operation stores a number of operations which can be executed atomically.
#[derive(Debug)]
pub struct WriteOperation {
    pub object_name: String,
    /// flags are set by calling LIBRADOS_OPERATION_NOFLAG | LIBRADOS_OPERATION_ORDER_READS_WRITES
    /// all the other flags are documented in rados.rs
    pub flags: u32,
    pub mtime: i64,
    write_op_handle: rados_write_op_t,
}

impl Drop for WriteOperation {
    fn drop(&mut self) {
        unsafe {
            rados_release_write_op(self.write_op_handle);
        }
    }
}

/// A rados object extended attribute with name and value.
/// Can be iterated over
#[derive(Debug)]
pub struct XAttr {
    pub name: String,
    pub value: String,
    iter: rados_xattrs_iter_t,
}

/// The version of the librados library.
#[derive(Debug)]
pub struct RadosVersion {
    pub major: i32,
    pub minor: i32,
    pub extra: i32,
}

/// Connect to a Ceph cluster and return a connection handle rados_t
pub fn connect_to_ceph(user_id: &str, config_file: &str) -> Result<rados_t, RadosError> {
    let connect_id = try!(CString::new(user_id));
    let conf_file = try!(CString::new(config_file));
    unsafe {
        let mut cluster_handle: rados_t = ptr::null_mut();
        let ret_code = rados_create(&mut cluster_handle, connect_id.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        let ret_code = rados_conf_read_file(cluster_handle, conf_file.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        let ret_code = rados_connect(cluster_handle);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        Ok(cluster_handle)
    }
}

/// Disconnect from a Ceph cluster and destroy the connection handle rados_t
/// For clean up, this is only necessary after connect_to_ceph() has succeeded.
pub fn disconnect_from_ceph(cluster: rados_t) {
    if cluster.is_null() {
        // No need to do anything
        return;
    }
    unsafe {
        rados_shutdown(cluster);
    }
}

/// Create an io context. The io context allows you to perform operations within a particular pool.
/// For more details see rados_ioctx_t.
pub fn get_rados_ioctx(cluster: rados_t, pool_name: &str) -> Result<rados_ioctx_t, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let pool_name_str = try!(CString::new(pool_name));
    unsafe {
        let mut ioctx: rados_ioctx_t = ptr::null_mut();
        let ret_code = rados_ioctx_create(cluster, pool_name_str.as_ptr(), &mut ioctx);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        Ok(ioctx)
    }
}

/// Create an io context. The io context allows you to perform operations within a particular pool.
/// For more details see rados_ioctx_t.
pub fn get_rados_ioctx2(cluster: rados_t, pool_id: i64) -> Result<rados_ioctx_t, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    unsafe {
        let mut ioctx: rados_ioctx_t = ptr::null_mut();
        let ret_code = rados_ioctx_create2(cluster, pool_id, &mut ioctx);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        Ok(ioctx)
    }
}

/// This just tells librados that you no longer need to use the io context.
/// It may not be freed immediately if there are pending asynchronous requests on it, but you
/// should not use an io context again after calling this function on it.
/// This does not guarantee any asynchronous writes have completed. You must call rados_aio_flush()
/// on the io context before destroying it to do that.
pub fn destroy_rados_ioctx(ctx: rados_ioctx_t) {
    if ctx.is_null() {
        // No need to do anything
        return;
    }
    unsafe {
        rados_ioctx_destroy(ctx);
    }
}

pub fn rados_stat_pool(ctx: rados_ioctx_t) -> Result<Struct_rados_pool_stat_t, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let mut pool_stat = Struct_rados_pool_stat_t::default();
    unsafe {
        let ret_code = rados_ioctx_pool_stat(ctx, &mut pool_stat);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        Ok(pool_stat)
    }
}

pub fn rados_pool_set_auid(ctx: rados_ioctx_t, auid: u64) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    unsafe {
        let ret_code = rados_ioctx_pool_set_auid(ctx, auid);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        return Ok(());
    }
}

pub fn rados_pool_get_auid(ctx: rados_ioctx_t) -> Result<u64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let mut auid: u64 = 0;
    unsafe {
        let ret_code = rados_ioctx_pool_get_auid(ctx, &mut auid);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        return Ok(auid);
    }
}

/// Test whether the specified pool requires alignment or not.
pub fn rados_pool_requires_alignment(ctx: rados_ioctx_t) -> Result<bool, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    unsafe {
        let ret_code = rados_ioctx_pool_requires_alignment(ctx);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        if ret_code == 0 {
            return Ok(false);
        } else {
            return Ok(true);
        }
    }
}

/// Get the alignment flavor of a pool
pub fn rados_pool_required_alignment(ctx: rados_ioctx_t) -> Result<u64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    unsafe {
        let ret_code = rados_ioctx_pool_required_alignment(ctx);
        return Ok(ret_code);
    }
}

/// Get the pool id of the io context
pub fn rados_object_get_id(ctx: rados_ioctx_t) -> Result<i64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    unsafe {
        let pool_id = rados_ioctx_get_id(ctx);
        Ok(pool_id)
    }

}

/// Get the pool name of the io context
pub fn rados_get_pool_name(ctx: rados_ioctx_t) -> Result<String, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let mut buffer: Vec<u8> = Vec::with_capacity(500);

    unsafe {
        // length of string stored, or -ERANGE if buffer too small
        let ret_code = rados_ioctx_get_pool_name(ctx, buffer.as_mut_ptr() as *mut i8, buffer.capacity() as u32);
        if ret_code == -ERANGE {
            // Buffer was too small
            buffer.reserve(1000);
            buffer.set_len(1000);
            let ret_code = rados_ioctx_get_pool_name(ctx, buffer.as_mut_ptr() as *mut i8, buffer.capacity() as u32);
            if ret_code < 0 {
                return Err(RadosError::new(try!(get_error(ret_code as i32))));
            }
            return Ok(String::from_utf8_lossy(&buffer).into_owned());
        } else if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        } else {
            buffer.set_len(ret_code as usize);
            return Ok(String::from_utf8_lossy(&buffer).into_owned());
        }
    }

}

/// Set the key for mapping objects to pgs within an io context.
pub fn rados_locator_set_key(ctx: rados_ioctx_t, key: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let key_str = try!(CString::new(key));
    unsafe {
        rados_ioctx_locator_set_key(ctx, key_str.as_ptr());
    }
    Ok(())
}

/// Set the namespace for objects within an io context
/// The namespace specification further refines a pool into different domains.
/// The mapping of objects to pgs is also based on this value.
pub fn rados_set_namespace(ctx: rados_ioctx_t, namespace: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let namespace_str = try!(CString::new(namespace));
    unsafe {
        rados_ioctx_set_namespace(ctx, namespace_str.as_ptr());
    }
    Ok(())
}

/// Start listing objects in a pool
pub fn rados_list_pool_objects(ctx: rados_ioctx_t) -> Result<rados_list_ctx_t, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let mut rados_list_ctx: rados_list_ctx_t = ptr::null_mut();
    unsafe {
        let ret_code = rados_nobjects_list_open(ctx, &mut rados_list_ctx);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(rados_list_ctx)
}

/// Create a pool-wide snapshot
pub fn rados_snap_create(ctx: rados_ioctx_t, snap_name: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }

    let snap_name_str = try!(CString::new(snap_name));
    unsafe {
        let ret_code = rados_ioctx_snap_create(ctx, snap_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Delete a pool snapshot
pub fn rados_snap_remove(ctx: rados_ioctx_t, snap_name: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let snap_name_str = try!(CString::new(snap_name));

    unsafe {
        let ret_code = rados_ioctx_snap_remove(ctx, snap_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Rollback an object to a pool snapshot
/// The contents of the object will be the same as when the snapshot was taken.
pub fn rados_snap_rollback(ctx: rados_ioctx_t, object_name: &str, snap_name: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let snap_name_str = try!(CString::new(snap_name));
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_ioctx_snap_rollback(ctx, object_name_str.as_ptr(), snap_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Set the snapshot from which reads are performed.
/// Subsequent reads will return data as it was at the time of that snapshot.
pub fn rados_snap_set_read(ctx: rados_ioctx_t, snap_id: u64) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }

    unsafe {
        rados_ioctx_snap_set_read(ctx, snap_id);
    }
    Ok(())
}

/// Allocate an ID for a self-managed snapshot
/// Get a unique ID to put in the snaphot context to create a snapshot.
/// A clone of an object is not created until a write with the new snapshot context is completed.
pub fn rados_selfmanaged_snap_create(ctx: rados_ioctx_t) -> Result<u64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let mut snap_id: u64 = 0;
    unsafe {
        let ret_code = rados_ioctx_selfmanaged_snap_create(ctx, &mut snap_id);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(snap_id)
}

/// Remove a self-managed snapshot
/// This increases the snapshot sequence number, which will cause snapshots to be removed lazily.
pub fn rados_selfmanaged_snap_remove(ctx: rados_ioctx_t, snap_id: u64) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }

    unsafe {
        let ret_code = rados_ioctx_selfmanaged_snap_remove(ctx, snap_id);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Rollback an object to a self-managed snapshot
/// The contents of the object will be the same as when the snapshot was taken.
pub fn rados_selfmanaged_snap_rollback(ctx: rados_ioctx_t, object_name: &str, snap_id: u64) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_ioctx_selfmanaged_snap_rollback(ctx, object_name_str.as_ptr(), snap_id);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Set the snapshot context for use when writing to objects
/// This is stored in the io context, and applies to all future writes.
// pub fn rados_selfmanaged_snap_set_write_ctx(ctx: rados_ioctx_t) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
//
/// List all the ids of pool snapshots
// pub fn rados_snap_list(ctx: rados_ioctx_t, snaps: *mut rados_snap_t) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
// let mut buffer: Vec<u64> = Vec::with_capacity(500);
//
//
// unsafe {
// let ret_code = rados_ioctx_snap_list(ctx, &mut buffer, buffer.capacity());
// if ret_code == -ERANGE {
// }
// if ret_code < 0 {
// return Err(RadosError::new(try!(get_error(ret_code as i32))));
// }
// }
// Ok(buffer)
// }
//
/// Get the id of a pool snapshot
pub fn rados_snap_lookup(ctx: rados_ioctx_t, snap_name: &str) -> Result<u64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let snap_name_str = try!(CString::new(snap_name));
    let mut snap_id: u64 = 0;
    unsafe {
        let ret_code = rados_ioctx_snap_lookup(ctx, snap_name_str.as_ptr(), &mut snap_id);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(snap_id)
}

/// Get the name of a pool snapshot
pub fn rados_snap_get_name(ctx: rados_ioctx_t, snap_id: u64) -> Result<String, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }

    let out_buffer: Vec<u8> = Vec::with_capacity(500);
    let out_buff_size = out_buffer.capacity();
    let out_str = try!(CString::new(out_buffer));
    unsafe {
        let ret_code = rados_ioctx_snap_get_name(ctx, snap_id, out_str.as_ptr() as *mut i8, out_buff_size as i32);
        if ret_code == -ERANGE {
        }
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(out_str.to_string_lossy().into_owned())
}

/// Find when a pool snapshot occurred
pub fn rados_snap_get_stamp(ctx: rados_ioctx_t, snap_id: u64) -> Result<i64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }

    let mut time_id: i64 = 0;
    unsafe {
        let ret_code = rados_ioctx_snap_get_stamp(ctx, snap_id, &mut time_id);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(time_id)
}

/// Return the version of the last object read or written to.
/// This exposes the internal version number of the last object read or written via this io context
pub fn rados_get_object_last_version(ctx: rados_ioctx_t) -> Result<u64, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    unsafe {
        let obj_id = rados_get_last_version(ctx);
        Ok(obj_id)
    }
}

/// Write len bytes from buf into the oid object, starting at offset off.
/// The value of len must be <= UINT_MAX/2.
pub fn rados_object_write(ctx: rados_ioctx_t, object_name: &str, buffer: &[u8], offset: u64) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let obj_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_write(ctx, obj_name_str.as_ptr(), buffer.as_ptr() as *const i8, buffer.len(), offset);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// The object is filled with the provided data. If the object exists, it is atomically
/// truncated and then written.
pub fn rados_object_write_full(ctx: rados_ioctx_t, object_name: &str, buffer: &[u8]) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let obj_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_write_full(ctx, obj_name_str.as_ptr(), buffer.as_ptr() as *const i8, buffer.len());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Efficiently copy a portion of one object to another
/// If the underlying filesystem on the OSD supports it, this will be a copy-on-write clone.
/// The src and dest objects must be in the same pg. To ensure this, the io context should
/// have a locator key set (see rados_ioctx_locator_set_key()).
pub fn rados_object_clone_range(ctx: rados_ioctx_t, dst_object_name: &str, dst_offset: u64, src_object_name: &str,
                                src_offset: u64, length: usize)
                                -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let dst_name_str = try!(CString::new(dst_object_name));
    let src_name_str = try!(CString::new(src_object_name));

    unsafe {
        let ret_code =
            rados_clone_range(ctx, dst_name_str.as_ptr(), dst_offset, src_name_str.as_ptr(), src_offset, length);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Append len bytes from buf into the oid object.
pub fn rados_object_append(ctx: rados_ioctx_t, object_name: &str, buffer: &[u8]) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let obj_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_append(ctx, obj_name_str.as_ptr(), buffer.as_ptr() as *const i8, buffer.len());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Read data from an object.  This fills the slice given and returns the amount of bytes read
/// The io context determines the snapshot to read from, if any was set by
/// rados_ioctx_snap_set_read().
pub fn rados_object_read(ctx: rados_ioctx_t, object_name: &str, fill_buffer: &mut [u8], read_offset: u64)
                         -> Result<i32, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_read(ctx,
                                  object_name_str.as_ptr(),
                                  fill_buffer.as_mut_ptr() as *mut i8,
                                  fill_buffer.len(),
                                  read_offset);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
        Ok(ret_code)
    }
}

/// Delete an object
/// Note: This does not delete any snapshots of the object.
pub fn rados_object_remove(ctx: rados_ioctx_t, object_name: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_remove(ctx, object_name_str.as_ptr() as *const i8);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Resize an object
/// If this enlarges the object, the new area is logically filled with zeroes.
/// If this shrinks the object, the excess data is removed.
pub fn rados_object_trunc(ctx: rados_ioctx_t, object_name: &str, new_size: u64) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_trunc(ctx, object_name_str.as_ptr(), new_size);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Get the value of an extended attribute on an object.
pub fn rados_object_getxattr(ctx: rados_ioctx_t, object_name: &str, attr_name: &str, fill_buffer: &mut [u8])
                             -> Result<i32, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let attr_name_str = try!(CString::new(attr_name));

    unsafe {
        let ret_code = rados_getxattr(ctx,
                                      object_name_str.as_ptr() as *const i8,
                                      attr_name_str.as_ptr() as *const i8,
                                      fill_buffer.as_mut_ptr() as *mut i8,
                                      fill_buffer.len());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
        Ok(ret_code)
    }
}

/// Set an extended attribute on an object.
pub fn rados_object_setxattr(ctx: rados_ioctx_t, object_name: &str, attr_name: &str, attr_value: &mut [u8])
                             -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let attr_name_str = try!(CString::new(attr_name));

    unsafe {
        let ret_code = rados_setxattr(ctx,
                                      object_name_str.as_ptr() as *const i8,
                                      attr_name_str.as_ptr() as *const i8,
                                      attr_value.as_mut_ptr() as *mut i8,
                                      attr_value.len());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Delete an extended attribute from an object.
pub fn rados_object_rmxattr(ctx: rados_ioctx_t, object_name: &str, attr_name: &str) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let attr_name_str = try!(CString::new(attr_name));

    unsafe {
        let ret_code = rados_rmxattr(ctx, object_name_str.as_ptr() as *const i8, attr_name_str.as_ptr() as *const i8);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

impl XAttr {
    /// Creates a new XAttr.  Call rados_getxattrs to create the iterator for this struct
    pub fn new(iter: rados_xattrs_iter_t) -> XAttr {
        XAttr {
            name: String::new(),
            value: String::new(),
            iter: iter,
        }
    }
}

impl Iterator for XAttr {
    type Item = XAttr;

    fn next(&mut self) -> Option<Self::Item> {
        // max xattr name is 255 bytes from what I can find
        let mut name_buffer: Vec<u8> = Vec::with_capacity(255);
        // max xattr is 64Kb from what I can find
        let mut value_buffer: Vec<u8> = Vec::with_capacity(64 * 1024);
        let mut val_length: usize = 0;
        unsafe {
            let ret_code = rados_getxattrs_next(self.iter,
                                                name_buffer.as_mut_ptr() as *mut *const i8,
                                                value_buffer.as_mut_ptr() as *mut *const i8,
                                                &mut val_length);

            if ret_code < 0 {
                // Something failed, however Iterator doesn't return Result so we return None
                None
            }
            // end of iterator reached
            else if val_length == 0 {
                rados_getxattrs_end(self.iter);
                None
            } else {
                Some(XAttr {
                    name: String::from_utf8_lossy(&name_buffer).into_owned(),
                    value: String::from_utf8_lossy(&value_buffer).into_owned(),
                    iter: self.iter,
                })
            }
        }
    }
}

/// Get the rados_xattrs_iter_t reference to iterate over xattrs on an object
/// Used in conjuction with XAttr::new() to iterate.
pub fn rados_get_xattr_iterator(ctx: rados_ioctx_t, object_name: &str) -> Result<rados_xattrs_iter_t, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let mut xattr_iterator_handle: rados_xattrs_iter_t = ptr::null_mut();

    unsafe {
        let ret_code = rados_getxattrs(ctx, object_name_str.as_ptr(), &mut xattr_iterator_handle);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(xattr_iterator_handle)
}

/// Get object stats (size,SystemTime)
pub fn rados_object_stat(ctx: rados_ioctx_t, object_name: &str) -> Result<(u64, SystemTime), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let mut psize: u64 = 0;
    let mut time: i64 = 0;

    unsafe {
        let ret_code = rados_stat(ctx, object_name_str.as_ptr(), &mut psize, &mut time);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok((psize, (UNIX_EPOCH + Duration::from_secs(time as u64))))
}

/// Update tmap (trivial map)
pub fn rados_object_tmap_update(ctx: rados_ioctx_t, object_name: &str, update: TmapOperation)
                                -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let buffer = try!(update.serialize());
    unsafe {
        let ret_code = rados_tmap_update(ctx, object_name_str.as_ptr(), buffer.as_ptr() as *const i8, buffer.len());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

// pub fn rados_object_tmap_put(ctx: rados_ioctx_t, object_name: &str) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }

/// Fetch complete tmap (trivial map) object
pub fn rados_object_tmap_get(ctx: rados_ioctx_t, object_name: &str) -> Result<Vec<TmapOperation>, RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let mut buffer: Vec<u8> = Vec::with_capacity(500);

    unsafe {
        let ret_code = rados_tmap_get(ctx, object_name_str.as_ptr(), buffer.as_mut_ptr() as *mut i8, buffer.capacity());
        if ret_code == -ERANGE {
            buffer.reserve(1000);
            buffer.set_len(1000);
            let ret_code =
                rados_tmap_get(ctx, object_name_str.as_ptr(), buffer.as_mut_ptr() as *mut i8, buffer.capacity());
            if ret_code < 0 {
                return Err(RadosError::new(try!(get_error(ret_code as i32))));
            }
        } else if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    match TmapOperation::deserialize(&buffer) {
        IResult::Done(_, tmap) => Ok(tmap),
        IResult::Incomplete(needed) => {
            Err(RadosError::new(format!("deserialize of ceph tmap failed.
            Input from Ceph was too small.  Needed: {:?} more bytes", needed)))
        },
        IResult::Error(e) => Err(RadosError::new(e.to_string())),
    }
}

/// Execute an OSD class method on an object
/// The OSD has a plugin mechanism for performing complicated operations on an object atomically.
/// These plugins are called classes. This function allows librados users to call the custom
/// methods. The input and output formats are defined by the class. Classes in ceph.git can
/// be found in src/cls subdirectories
pub fn rados_object_exec(ctx: rados_ioctx_t, object_name: &str, class_name: &str, method_name: &str,
                         input_buffer: &[u8], output_buffer: &mut [u8])
                         -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let class_name_str = try!(CString::new(class_name));
    let method_name_str = try!(CString::new(method_name));

    unsafe {
        let ret_code = rados_exec(ctx,
                                  object_name_str.as_ptr(),
                                  class_name_str.as_ptr(),
                                  method_name_str.as_ptr(),
                                  input_buffer.as_ptr() as *const i8,
                                  input_buffer.len(),
                                  output_buffer.as_mut_ptr() as *mut i8,
                                  output_buffer.len());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}
// pub fn rados_object_watch(ctx: rados_ioctx_t, object_name: &str) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
// pub fn rados_object_watch2(ctx: rados_ioctx_t, object_name: &str) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
// pub fn rados_object_watch_check(ctx: rados_ioctx_t, cookie: u64) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
// pub fn rados_object_unwatch(ctx: rados_ioctx_t, object_name: &str) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
// pub fn rados_object_unwatch2(ctx: rados_ioctx_t, cookie: u64) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
//
/// Sychronously notify watchers of an object
/// This blocks until all watchers of the object have received and reacted to the notify, or a timeout is reached.
pub fn rados_object_notify(ctx: rados_ioctx_t, object_name: &str, data: &[u8]) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_notify(ctx, object_name_str.as_ptr(), 0, data.as_ptr() as *const i8, data.len() as i32);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}
// pub fn rados_object_notify2(ctx: rados_ioctx_t, object_name: &str) -> Result<(), RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
//
// unsafe {
// }
// }
//
/// Acknolwedge receipt of a notify
pub fn rados_object_notify_ack(ctx: rados_ioctx_t, object_name: &str, notify_id: u64, cookie: u64,
                               buffer: Option<&[u8]>)
                               -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    match buffer {
        Some(buf) => unsafe {
            let ret_code = rados_notify_ack(ctx,
                                            object_name_str.as_ptr(),
                                            notify_id,
                                            cookie,
                                            buf.as_ptr() as *const i8,
                                            buf.len() as i32);
            if ret_code < 0 {
                return Err(RadosError::new(try!(get_error(ret_code as i32))));
            }
        },
        None => unsafe {
            let ret_code = rados_notify_ack(ctx, object_name_str.as_ptr(), notify_id, cookie, ptr::null(), 0);
            if ret_code < 0 {
                return Err(RadosError::new(try!(get_error(ret_code as i32))));
            }

        },
    }
    Ok(())
}
/// Set allocation hint for an object
/// This is an advisory operation, it will always succeed (as if it was submitted with a
/// LIBRADOS_OP_FLAG_FAILOK flag set) and is not guaranteed to do anything on the backend.
pub fn rados_object_set_alloc_hint(ctx: rados_ioctx_t, object_name: &str, expected_object_size: u64,
                                   expected_write_size: u64)
                                   -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));

    unsafe {
        let ret_code = rados_set_alloc_hint(ctx, object_name_str.as_ptr(), expected_object_size, expected_write_size);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

// Perform a compound read operation synchronously
pub fn rados_perform_read_operations(read_op: ReadOperation, ctx: rados_ioctx_t) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(read_op.object_name.clone()));

    unsafe {
        let ret_code =
            rados_read_op_operate(read_op.read_op_handle, ctx, object_name_str.as_ptr(), read_op.flags as i32);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

// Perform a compound write operation synchronously
pub fn rados_commit_write_operations(write_op: &mut WriteOperation, ctx: rados_ioctx_t) -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(write_op.object_name.clone()));

    unsafe {
        let ret_code = rados_write_op_operate(write_op.write_op_handle,
                                              ctx,
                                              object_name_str.as_ptr(),
                                              &mut write_op.mtime,
                                              write_op.flags as i32);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Take an exclusive lock on an object.
pub fn rados_object_lock_exclusive(ctx: rados_ioctx_t, object_name: &str, lock_name: &str, cookie_name: &str,
                                   description: &str, duration_time: &mut timeval, lock_flags: u8)
                                   -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let lock_name_str = try!(CString::new(lock_name));
    let cookie_name_str = try!(CString::new(cookie_name));
    let description_str = try!(CString::new(description));

    unsafe {
        let ret_code = rados_lock_exclusive(ctx,
                                            object_name_str.as_ptr(),
                                            lock_name_str.as_ptr(),
                                            cookie_name_str.as_ptr(),
                                            description_str.as_ptr(),
                                            duration_time,
                                            lock_flags);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Take a shared lock on an object.
pub fn rados_object_lock_shared(ctx: rados_ioctx_t, object_name: &str, lock_name: &str, cookie_name: &str,
                                description: &str, tag_name: &str, duration_time: &mut timeval, lock_flags: u8)
                                -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let lock_name_str = try!(CString::new(lock_name));
    let cookie_name_str = try!(CString::new(cookie_name));
    let description_str = try!(CString::new(description));
    let tag_name_str = try!(CString::new(tag_name));

    unsafe {
        let ret_code = rados_lock_shared(ctx,
                                         object_name_str.as_ptr(),
                                         lock_name_str.as_ptr(),
                                         cookie_name_str.as_ptr(),
                                         tag_name_str.as_ptr(),
                                         description_str.as_ptr(),
                                         duration_time,
                                         lock_flags);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// Release a shared or exclusive lock on an object.
pub fn rados_object_unlock(ctx: rados_ioctx_t, object_name: &str, lock_name: &str, cookie_name: &str)
                           -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let lock_name_str = try!(CString::new(lock_name));
    let cookie_name_str = try!(CString::new(cookie_name));

    unsafe {
        let ret_code = rados_unlock(ctx, object_name_str.as_ptr(), lock_name_str.as_ptr(), cookie_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

/// List clients that have locked the named object lock and information about the lock.
/// The number of bytes required in each buffer is put in the corresponding size out parameter.
/// If any of the provided buffers are too short, -ERANGE is returned after these sizes are filled in.
// pub fn rados_object_list_lockers(ctx: rados_ioctx_t, object_name: &str, lock_name: &str, exclusive: u8, ) ->
// Result<isize, RadosError> {
// if ctx.is_null() {
// return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
// }
// let object_name_str = try!(CString::new(object_name));
//
// unsafe {
// let ret_code = rados_list_lockers(ctx,
// o: *const ::libc::c_char,
// name: *const ::libc::c_char,
// exclusive: *mut ::libc::c_int,
// tag: *mut ::libc::c_char,
// tag_len: *mut size_t,
// clients: *mut ::libc::c_char,
// clients_len: *mut size_t,
// cookies: *mut ::libc::c_char,
// cookies_len: *mut size_t,
// addrs: *mut ::libc::c_char,
// addrs_len: *mut size_t);
// }
// }
/// Releases a shared or exclusive lock on an object, which was taken by the specified client.
pub fn rados_object_break_lock(ctx: rados_ioctx_t, object_name: &str, lock_name: &str, client_name: &str,
                               cookie_name: &str)
                               -> Result<(), RadosError> {
    if ctx.is_null() {
        return Err(RadosError::new("Rados ioctx not created.  Please initialize first".to_string()));
    }
    let object_name_str = try!(CString::new(object_name));
    let lock_name_str = try!(CString::new(lock_name));
    let cookie_name_str = try!(CString::new(cookie_name));
    let client_name_str = try!(CString::new(client_name));

    unsafe {
        let ret_code = rados_break_lock(ctx,
                                        object_name_str.as_ptr(),
                                        lock_name_str.as_ptr(),
                                        client_name_str.as_ptr(),
                                        cookie_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}

pub fn rados_blacklist_client(cluster: rados_t, client: IpAddr, expire_seconds: u32) -> Result<(), RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let client_address = try!(CString::new(client.to_string()));
    unsafe {
        let ret_code = rados_blacklist_add(cluster, client_address.as_ptr() as *mut i8, expire_seconds);

        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
    Ok(())
}


/// Returns back a collection of Rados Pools
///
/// pool_buffer should be allocated with:
/// ```
/// let pool_buffer: Vec<u8> = Vec::with_capacity(<whatever size>);
/// ```
/// buf_size should be the value used with_capacity
///
/// Returns Ok(Vec<String>) - A list of Strings of the pool names.
///
#[allow(unused_variables)]
pub fn rados_pools(cluster: rados_t) -> Result<Vec<String>, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let mut pools: Vec<String> = Vec::new();
    let pool_slice: &[u8];
    let mut pool_buffer: Vec<u8> = Vec::with_capacity(500);

    unsafe {
        let len = rados_pool_list(cluster, pool_buffer.as_mut_ptr() as *mut i8, pool_buffer.capacity());
        if len > pool_buffer.capacity() as i32 {
            // rados_pool_list requires more buffer than we gave it
            pool_buffer.reserve(len as usize);
            let len = rados_pool_list(cluster, pool_buffer.as_mut_ptr() as *mut i8, pool_buffer.capacity());
            // Tell the Vec how much Ceph read into the buffer
            pool_buffer.set_len(len as usize);
        } else {
            // Tell the Vec how much Ceph read into the buffer
            pool_buffer.set_len(len as usize);
        }
    }
    let mut cursor = Cursor::new(&pool_buffer);
    loop {
        let mut string_buf: Vec<u8> = Vec::new();
        let read = try!(cursor.read_until(0x00, &mut string_buf));
        if read == 0 {
            // End of the pool_buffer;
            break;
        } else if read == 1 {
            // Read a double \0.  Time to break
            break;
        } else {
            // Read a String
            pools.push(String::from_utf8_lossy(&string_buf).into_owned());
        }
    }

    Ok(pools)
}

/// Create a pool with default settings
/// The default owner is the admin user (auid 0). The default crush rule is rule 0.
pub fn rados_create_pool(cluster: rados_t, pool_name: &str) -> Result<(), RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let pool_name_str = try!(CString::new(pool_name));
    unsafe {
        let ret_code = rados_pool_create(cluster, pool_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
    }
    return Ok(());
}
/// Delete a pool and all data inside it
/// The pool is removed from the cluster immediately, but the actual data is deleted in
/// the background.
pub fn rados_delete_pool(cluster: rados_t, pool_name: &str) -> Result<(), RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let pool_name_str = try!(CString::new(pool_name));
    unsafe {
        let ret_code = rados_pool_delete(cluster, pool_name_str.as_ptr());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
    }
    return Ok(());
}

/// Lookup a Ceph pool id.  If the pool doesn't exist it will return Ok(None).
pub fn rados_lookup_pool(cluster: rados_t, pool_name: &str) -> Result<Option<i64>, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let pool_name_str = try!(CString::new(pool_name));
    unsafe {
        let ret_code: i64 = rados_pool_lookup(cluster, pool_name_str.as_ptr());
        if ret_code >= 0 {
            return Ok(Some(ret_code));
        } else if ret_code as i32 == -ENOENT {
            return Ok(None);
        } else {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        }
    }
}

pub fn rados_reverse_lookup_pool(cluster: rados_t, pool_id: i64) -> Result<String, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let mut buffer: Vec<u8> = Vec::with_capacity(500);

    unsafe {
        let ret_code = rados_pool_reverse_lookup(cluster, pool_id, buffer.as_mut_ptr() as *mut i8, buffer.capacity());
        if ret_code == -ERANGE {
            // Buffer was too small
            buffer.reserve(1000);
            buffer.set_len(1000);
            let ret_code =
                rados_pool_reverse_lookup(cluster, pool_id, buffer.as_mut_ptr() as *mut i8, buffer.capacity());
            if ret_code < 0 {
                return Err(RadosError::new(try!(get_error(ret_code as i32))));
            }
            return Ok(String::from_utf8_lossy(&buffer).into_owned());
        } else if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code as i32))));
        } else {
            return Ok(String::from_utf8_lossy(&buffer).into_owned());
        }
    }
}


/// Get the version of librados.
pub fn rados_libversion() -> RadosVersion {
    let mut major: c_int = 0;
    let mut minor: c_int = 0;
    let mut extra: c_int = 0;
    unsafe {
        rados_version(&mut major, &mut minor, &mut extra);
    }
    return RadosVersion {
        major: major,
        minor: minor,
        extra: extra,
    };
}

/// Read usage info about the cluster
/// This tells you total space, space used, space available, and number of objects.
/// These are not updated immediately when data is written, they are eventually consistent.
pub fn rados_stat_cluster(cluster: rados_t) -> Result<Struct_rados_cluster_stat_t, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let mut cluster_stat = Struct_rados_cluster_stat_t::default();
    unsafe {
        let ret_code = rados_cluster_stat(cluster, &mut cluster_stat);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
    }

    return Ok(cluster_stat);
}


pub fn rados_fsid(cluster: rados_t) -> Result<Uuid, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let mut fsid_buffer: Vec<u8> = Vec::with_capacity(37);
    let fsid: Uuid;
    unsafe {
        let ret_code = rados_cluster_fsid(cluster, fsid_buffer.as_mut_ptr() as *mut i8, fsid_buffer.capacity());
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
        if ret_code > 0 {
            // Length of buffer required for fsid
            fsid_buffer.reserve(ret_code as usize);
            rados_cluster_fsid(cluster, fsid_buffer.as_mut_ptr() as *mut i8, fsid_buffer.capacity());
            // Tell the Vec how much Ceph read into the buffer
            fsid_buffer.set_len(ret_code as usize);
        }
    }
    fsid = try!(Uuid::from_bytes(&fsid_buffer));
    Ok(fsid)
}

/// Ping a monitor to assess liveness
/// May be used as a simply way to assess liveness, or to obtain
/// information about the monitor in a simple way even in the
/// absence of quorum.
pub fn ping_monitor(cluster: rados_t, mon_id: &str) -> Result<String, RadosError> {
    if cluster.is_null() {
        return Err(RadosError::new("Rados not connected.  Please initialize cluster".to_string()));
    }
    let mon_id_str = try!(CString::new(mon_id));
    let out_buffer: Vec<u8> = Vec::with_capacity(500);
    let out_buff_size = out_buffer.capacity();
    let out_str = try!(CString::new(out_buffer));
    unsafe {
        let ret_code = rados_ping_monitor(cluster,
                                          mon_id_str.as_ptr(),
                                          out_str.as_ptr() as *mut *mut i8,
                                          out_buff_size as *mut usize);
        if ret_code < 0 {
            return Err(RadosError::new(try!(get_error(ret_code))));
        }
    }
    Ok(out_str.to_string_lossy().into_owned())

}
