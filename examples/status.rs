extern crate ceph;

use ceph::CephClient;

pub fn main() {
    let ceph_client = CephClient::new("admin", "/etc/ceph/ceph.conf").unwrap();
    println!("Status: {}", ceph_client.status().unwrap());
}
