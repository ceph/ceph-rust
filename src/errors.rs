use ceph_rust::error as ceph_error;
use CephVersion;

error_chain!{
    errors {
            Parse(input: String) {
                description("An error occurred during parsing")
                display("Couldn't parse the CephVersion from {}", input)
            }
            MinVersion(min: CephVersion, current_version: CephVersion) {
                description("Ceph version is too low")
                display("{:?} minimum, your version is {:?}", min, current_version)
            }
            Rados(error: String) {
                description("Rados error")
                display("Rados Error: {:?}", error)
            }
    }
    foreign_links {
        RadosClient(ceph_error::RadosError) #[doc = "Ceph Client Error"];
    }
}
