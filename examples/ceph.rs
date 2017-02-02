// Copyright 2017 LambdaStack All rights reserved.
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
extern crate libc;

use libc::*;
use std::ffi::{CStr, CString};
use std::{ptr, str, slice};

#[cfg(target_os = "linux")]
use ceph_rust::ceph as ceph_helpers;
#[cfg(target_os = "linux")]
use ceph_rust::rados as ceph;
#[cfg(target_os = "linux")]
use ceph_rust::admin_sockets::*;

macro_rules! zeroed_c_char_buf {
	($n:expr) => {
		repeat(0).take($n).collect::<Vec<c_char>>();
	}
}

#[cfg(not(target_os = "linux"))]
fn main() {}

// NB: The examples below show a mix of raw native access and rust specific calls.

#[cfg(target_os = "linux")]
fn main() {
    let mut major: i32 = 0;
    let mut minor: i32 = 0;
    let mut extra: i32 = 0;

    let config_file = CString::new("/etc/ceph/ceph.conf").unwrap();
    let pool_name = CString::new("lsio").unwrap();
    let mut cluster: ceph::rados_t = std::ptr::null_mut();
    let mut ret_code: i32;

    // NB: These examples (except for a few) are low level examples that require the unsafe block.
    // However, work for the higher level pur Rust is being worked on in the ceph.rs module of
    // the library. A few of those are present below. We will create a common Result or Option
    // return and allow for pattern matching.

    // Example of accessing the `Admin Socket` for mon
    match admin_socket_command("help", "/var/run/ceph/ceph-mon.ceph-vm1.asok") {
        Ok(json) => {
            println!("{}", json);
        },
        Err(e) => { println!("{}", e); },
    }

    unsafe {
        ceph::rados_version(&mut major, &mut minor, &mut extra);
        ret_code = ceph::rados_create(&mut cluster, std::ptr::null());
        println!("Return code: {} - {:?}", ret_code, cluster);

        ret_code = ceph::rados_conf_read_file(cluster, config_file.as_ptr());
        println!("Return code: {} - {:?}", ret_code, cluster);

        ret_code = ceph::rados_connect(cluster);
        println!("Return code: {} - {:?}", ret_code, cluster);

        ret_code = ceph::rados_pool_create(cluster, pool_name.as_ptr());
        println!("Return code: {}", ret_code);

        let pools_list = ceph_helpers::rados_pools(cluster).unwrap();
        println!("{:?}", pools_list);

        ret_code = ceph::rados_pool_delete(cluster, pool_name.as_ptr());
        println!("Return code: {}", ret_code);

        let instance_id = ceph::rados_get_instance_id(cluster);
        println!("Instance ID: {}", instance_id);

        let buf_size: usize = 37; // 36 is the constant size +1 for null.
        let mut fs_id: Vec<u8> = Vec::with_capacity(buf_size);

        let len = ceph::rados_cluster_fsid(cluster, fs_id.as_mut_ptr() as *mut i8, buf_size);
        let slice = slice::from_raw_parts(fs_id.as_mut_ptr(), buf_size - 1);
        let s: &str = str::from_utf8(slice).unwrap();
        println!("rados_cluster_fsid len: {} - {}", len, s);

        // Rust specific example...
        let cluster_stat = ceph_helpers::rados_stat_cluster(cluster);
        println!("Cluster stat: {:?}", cluster_stat);

        // Mon command to check the health. Same as `ceph -s`
        match ceph_helpers::ceph_mon_command(cluster, "{\"prefix\": \"status\"}") {
            Ok((outbuf, outs)) => {
                match outbuf {
                    Some(output) => println!("Ceph mon command (outbuf):\n{}", output),
                    None => {},
                }
                match outs {
                    Some(output) => println!("Ceph mon command (outs):\n{}", output),
                    None => {},
                }
            },
            Err(e) => {println!("{:?}", e);},
        }

        // Currently - parses the `ceph --version` call. The admin socket commands `version` and `git_version`
        // will be called soon to replace the string parse.
        let ceph_ver = ceph_helpers::ceph_version();
        println!("Ceph Version - {:?}", ceph_ver);

        ceph::rados_shutdown(cluster);
    }

    println!("RADOS Version - v{}.{}.{}", major, minor, extra);

}
