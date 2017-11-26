extern crate ceph_client;

use ceph_client::CephClient;

pub fn main() {
    let ceph_client = CephClient::new("admin", "/etc/ceph/ceph.conf").unwrap();
    println!("Status: {}", ceph_client.status().unwrap());
}