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

extern crate ceph;
extern crate libc;

use ceph::JsonData;
#[cfg(target_os = "linux")]
use ceph::admin_sockets::*;
#[cfg(target_os = "linux")]
use ceph::ceph as ceph_helpers;
#[cfg(target_os = "linux")]
use ceph::rados;

#[cfg(not(target_os = "linux"))]
fn main() {}

// NB: The examples below show a mix of raw native access and rust specific calls.

#[cfg(target_os = "linux")]
fn main() {
    let pool_name = "lsio";
    // NB: These examples (except for a few) are low level examples that require the unsafe block.
    // However, work for the higher level pur Rust is being worked on in the ceph.rs module of
    // the library. A few of those are present below. We will create a common Result or Option
    // return and allow for pattern matching.

    // Example of accessing the `Admin Socket` for mon
    match admin_socket_command("help", "/var/run/ceph/ceph-mon.ip-172-31-31-247.asok") {
        Ok(json) => {
            println!("{}", json);
        },
        Err(e) => {
            println!("{}", e);
        },
    }

    let rados_version = ceph_helpers::rados_libversion();
    println!("Librados version: {:?}", rados_version);

    println!("Connecting to ceph");
    let cluster = ceph_helpers::connect_to_ceph("admin", "/etc/ceph/ceph.conf").unwrap();
    println!("Creating pool {}", pool_name);
    cluster.rados_create_pool(pool_name).unwrap();

    println!("Listing pools");
    let pools_list = cluster.rados_pools().unwrap();
    for pool in pools_list {
        println!("pool: {}", pool);
    }

    println!("Deleting pool: {}", pool_name);
    cluster.rados_delete_pool(pool_name).unwrap();

    println!("Getting cluster fsid");
    let fsid = cluster.rados_fsid();
    println!("rados_cluster_fsid {:?}", fsid);

    let cluster_stat = cluster.rados_stat_cluster().unwrap();
    println!("Cluster stat: {:?}", cluster_stat);

    // Print CephHealth of cluster
    println!("{}", cluster.ceph_health_string().unwrap_or("".to_string()));

    // Print Status of cluster health a different way
    println!("{}", cluster.ceph_status(&["health", "overall_status"]).unwrap());
    // Currently - parses the `ceph --version` call. The admin socket commands
    // `version` and `git_version`
    // will be called soon to replace the string parse.
    // Change to the real mon admin socket name
    let ceph_ver = ceph_helpers::ceph_version("/var/run/ceph/ceph-mon.ip-172-31-31-247.asok");
    println!("Ceph Version - {:?}", ceph_ver);
    // Mon command to check the health. Same as `ceph -s`
    match cluster.ceph_mon_command("prefix", "status", None) {
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
        Err(e) => {
            println!("{:?}", e);
        },
    }

    // This command encapsulates the lower level mon, osd, pg commands and returns JsonData
    // objects based on the key path
    println!("{:?}",
             cluster.ceph_command("prefix", "status", ceph_helpers::CephCommandTypes::Mon, &["health"]));

    // Get a list of Ceph librados commands in JSON format.
    // It's very long so it's commented out.
    // println!("{}", ceph_helpers::ceph_commands(cluster, None).unwrap().pretty());
    unsafe {
        println!("Getting rados instance id");
        let instance_id = rados::rados_get_instance_id(*cluster.inner());
        println!("Instance ID: {}", instance_id);
    }

    let fsid = cluster.rados_fsid().unwrap();
    println!("rados_cluster_fsid: {}", fsid.to_hyphenated().to_string());

    let ping_monitor = cluster.ping_monitor("ceph-mon.ceph-vm1"); // Change to support your mon name
    println!("Ping monitor: {:?}", ping_monitor);

    // Rust specific example...
    let cluster_stat = cluster.rados_stat_cluster();
    println!("Cluster stat: {:?}", cluster_stat);

    // Mon command to check the health. Same as `ceph -s`
    match cluster.ceph_mon_command("prefix", "status", None) {
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

    // Print CephHealth of cluster
    println!("{}", cluster.ceph_health_string().unwrap_or("".to_string()));

    // Print Status of cluster health a different way
    println!("{}", cluster.ceph_status(&["health", "overall_status"]).unwrap());

    // This command encapsulates the lower level mon, osd, pg commands and returns JsonData objects based on the key path
    println!("{:?}", cluster.ceph_command("prefix", "status", ceph_helpers::CephCommandTypes::Mon, &["health"]));

    // Get a list of Ceph librados commands in JSON format.
    // It's very long so it's commented out.
    // println!("{}", ceph_helpers::ceph_commands(cluster, None).unwrap().pretty());

    // Currently - parses the `ceph --version` call. The admin socket commands `version` and `git_version`
    // will be called soon to replace the string parse.
    let ceph_ver = ceph_helpers::ceph_version("/var/run/ceph/ceph-mon.ceph-vm1.asok"); // Change to the real mon admin socket name
    println!("Ceph Version - {:?}", ceph_ver);

    println!(
        "RADOS Version - v{}.{}.{}",
        rados_version.major, rados_version.minor, rados_version.extra
    );
}
