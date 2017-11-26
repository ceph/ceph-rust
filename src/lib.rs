// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

extern crate ceph_rust;

mod ceph_client;
mod errors;

pub use errors::*;

pub use ceph_client::CephClient;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

