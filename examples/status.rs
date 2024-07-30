extern crate ceph;

use ceph::{ceph::connect_to_ceph, error::RadosError, CephClient};

#[tokio::main]
pub async fn main() -> Result<(), RadosError> {
    let user_id = "admin".to_string();
    let config_file = "/etc/ceph/ceph.conf".to_string();
    // let ceph_client = CephClient::new(&user_id, &config_file)?;
    // println!("Status: {}", ceph_client.status().unwrap());

    let rados = match connect_to_ceph(&user_id.as_ref(), &config_file.as_ref()) {
        Ok(rados_t) => rados_t,
        Err(e) => return Err(e),
    };

    let pools = rados.rados_pools().await?;
    println!("pools: {:?}", pools);

    return Ok(());
}
