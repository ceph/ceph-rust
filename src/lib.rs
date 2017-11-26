// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

extern crate ceph_rust;

mod ceph_choices;
mod ceph_client;
mod ceph_types;
mod ceph_version;
pub mod errors;

// use errors::*;

pub use ceph_choices::CephChoices;
pub use ceph_client::CephClient;
pub use ceph_version::CephVersion;
pub use ceph_types::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
