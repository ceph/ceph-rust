use ceph_rust::error as ceph_error;

error_chain!{
    foreign_links {
        Rados(ceph_error::RadosError) #[doc = "Ceph Client Error"];
    }
}