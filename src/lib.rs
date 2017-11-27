// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;
extern crate libc;
#[macro_use]
extern crate log;
extern crate ceph_rust;
// #[macro_use]
// extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod ceph_client;
mod ceph_helpers;
mod ceph_types;
mod ceph_version;
mod mon_command;
pub mod errors;

// use errors::*;

pub use ceph_client::CephClient;
pub use ceph_version::CephVersion;
pub use ceph_types::*;
pub use mon_command::MonCommand;
pub use ceph_rust::cmd::OsdOption;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
