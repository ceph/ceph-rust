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

#![allow(unused_imports)]

extern crate ceph_rust;
extern crate lsio;
extern crate libc;

use std::iter::FromIterator;
use std::ptr;
use std::str;
use std::slice;
use std::ffi::{CStr, CString};

use libc::*;
#[cfg(target_os = "linux")]
use ceph_rust::rados as ceph;
#[cfg(target_os = "linux")]
use ceph_rust::helpers as ceph_helpers;

macro_rules! zeroed_c_char_buf {
	($n:expr) => {
		repeat(0).take($n).collect::<Vec<c_char>>();
	}
}

#[cfg(not(target_os = "linux"))]
fn main(){}

#[cfg(target_os = "linux")]
fn main() {
  let mut major: i32 = 0;
  let mut minor: i32 = 0;
  let mut extra: i32 = 0;

  let config_file = CString::new("/etc/ceph/ceph.conf").unwrap();
  let mut cluster: ceph::rados_t = std::ptr::null_mut();
  let mut ret_code: i32;

  unsafe {
    ceph::rados_version(&mut major, &mut minor, &mut extra);
    ret_code = ceph::rados_create(&mut cluster, std::ptr::null());
    println!("Return code: {} - {:?}", ret_code, cluster);

    ret_code = ceph::rados_conf_read_file(cluster, config_file.as_ptr());
    println!("Return code: {} - {:?}", ret_code, cluster);

    ret_code = ceph::rados_connect(cluster);
    println!("Return code: {} - {:?}", ret_code, cluster);

    let pools_list = ceph_helpers::rados_pools(cluster).unwrap();
    println!("{:?}", pools_list);

    ceph::rados_shutdown(cluster);
  }

  println!("v{}.{}.{}", major, minor, extra);

}
