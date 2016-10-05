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

use libc::{c_int, strerror};
use rados::*;

use std::error::Error as err;
use std::ffi::{CString, IntoStringError, NulError};
use std::mem;
use std::io::Cursor;
use std::io::Error;
use std::io::BufRead;
use std::string::FromUtf8Error;

/// Custom error handling for the library
#[derive(Debug)]
pub enum RadosError {
    FromUtf8Error(FromUtf8Error),
    NulError(NulError),
    Error(String),
    IoError(Error),
    IntoStringError(IntoStringError),
}

impl RadosError {
    /// Create a new RadosError with a String message
    fn new(err: String) -> RadosError {
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
        }
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

/// Connect to a Ceph cluster and return a connection handle rados_t
pub fn connect_to_ceph(user_id: &str, config_file: &str) -> Result<rados_t, RadosError> {
    let connect_id = try!(CString::new(user_id));
    let conf_file = try!(CString::new(config_file));
    unsafe {
        let mut cluster_handle: rados_t = mem::uninitialized();
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
