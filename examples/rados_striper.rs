#[cfg(feature = "rados_striper")]
use {ceph::ceph as ceph_helpers, ceph::error::RadosError, nix::errno::Errno, std::env, std::str};

#[cfg(not(feature = "rados_striper"))]
fn main() {}

#[cfg(feature = "rados_striper")]
fn main() {
    let user_id = "admin";
    let config_file = env::var("CEPH_CONF").unwrap_or("/etc/ceph/ceph.conf".to_string());
    let pool_name = "ceph-rust-test";

    println!("Connecting to ceph");
    let cluster = ceph_helpers::connect_to_ceph(user_id, &config_file).unwrap();

    println!("Creating pool {}", pool_name);
    match cluster.rados_create_pool(pool_name) {
        Ok(_) => {}
        Err(RadosError::ApiError(Errno::EEXIST)) => {
            cluster.rados_delete_pool(pool_name).unwrap();
            cluster.rados_create_pool(pool_name).unwrap();
        }
        Err(err) => panic!("{:?}", err),
    }

    let object_name = "ceph-rust-test-object";

    {
        println!("Creating ioctx");
        let ioctx = cluster.get_rados_ioctx(pool_name).unwrap();

        println!("Creating rados striper");
        let rados_striper = ioctx.get_rados_striper().unwrap();

        println!("Writing test object");
        rados_striper
            .rados_object_write(object_name, "lorem".as_bytes(), 0)
            .unwrap();
        rados_striper
            .rados_object_write(object_name, " ipsum".as_bytes(), 5)
            .unwrap();
    }

    {
        println!("Creating ioctx");
        let ioctx = cluster.get_rados_ioctx(pool_name).unwrap();

        println!("Creating rados striper");
        let rados_striper = ioctx.get_rados_striper().unwrap();

        println!("Getting test object stat");
        let (size, _) = rados_striper.rados_object_stat(object_name).unwrap();

        let mut read_buf = vec![0; size as usize];

        println!("Reading test object");
        rados_striper
            .rados_object_read(object_name, &mut read_buf, 0)
            .unwrap();

        let read_buf_str = str::from_utf8(&read_buf).unwrap();

        assert_eq!(read_buf_str, "lorem ipsum");
    }

    println!("Deleting pool {}", pool_name);
    cluster.rados_delete_pool(pool_name).unwrap();
}
