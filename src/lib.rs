// Copyright 2016 LambdaStack All rights reserved.
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

// NOTE: This attribute only needs to be set once.
#![doc(
    html_logo_url = "https://lambdastackio.github.io/static/images/lambdastack-200x200.png",
    html_favicon_url = "https://lambdastackio.github.io/static/images/favicon.ico",
    html_root_url = "https://lambdastackio.github.io/aws-sdk-rust/ceph-rust/ceph_rust/index.html"
)]

//! Ceph-rust is a thin layer over the librados C interface. A little higher
//! abstraction layer will
//! be coming next that will encapsulate all of the "C" specific features so
//! that only pure Rust will be needed.
//!
//! Only works on Linux
//! The documentation for librados can be found:
//! http://docs.ceph.com/docs/master/rados/api/librados/
//!
//! By default Ceph names librados as the following for the given platforms:
//! Hammer release:
//! RHEL/CentOS:
//! /usr/lib64/librados.so.2.0.0
//!
//! Ubuntu:
//! /usr/lib/librados.so.2.0.0
//!
//! You will need to do a symlink of the above link to the following:
//! RHEL/CentOS:
//! sudo ln -s /usr/lib64/librados.so.2.0.0 /usr/lib64/librados.so
//!
//! Ubuntu:
//! sudo ln -s /usr/lib/librados.so.2.0.0 /usr/lib/librados.so
//!
//! NOTE: If someone know of another way for Rust to find the librados file
//! then please issue
//! a PR for it. Thanks!
//!
//! See the /examples/ceph.rs for how to use the library.

#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate libc;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate nom;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

pub mod admin_sockets;
pub mod ceph;
pub mod ceph_volume;
pub mod cmd;
pub mod error;
pub mod json;
pub mod rados;
#[cfg(feature = "rados_striper")]
pub mod rados_striper;
pub mod status;
pub mod utils;

mod ceph_client;
mod ceph_version;
mod mon_command;

pub use crate::ceph_client::CephClient;
pub use crate::ceph_version::CephVersion;
pub use crate::cmd::{OsdOption, PoolOption};
pub use crate::mon_command::MonCommand;

pub type JsonData = serde_json::Value;
pub type JsonValue = serde_json::Value;
