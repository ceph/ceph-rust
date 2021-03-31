// Copyright 2017 LambdaStack All rights reserved.
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
extern crate serde_json;

use crate::ceph_version::CephVersion;
use serde_json::error::Error as SerdeJsonError;
use std::error::Error as StdError;
use std::ffi::{IntoStringError, NulError};
use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::{fmt, str::ParseBoolError};
use uuid::Error as UuidError;

extern crate nix;

/// Custom error handling for the library
#[derive(Debug)]
pub enum RadosError {
    FromUtf8Error(FromUtf8Error),
    NulError(NulError),
    Error(String),
    IoError(Error),
    ApiError(nix::errno::Errno),
    IntoStringError(IntoStringError),
    ParseIntError(ParseIntError),
    ParseBoolError(ParseBoolError),
    UuidError(UuidError),
    SerdeError(SerdeJsonError),
    /// This should be the minimum version and the current version
    MinVersion(CephVersion, CephVersion),
    Parse(String),
}

pub type RadosResult<T> = Result<T, RadosError>;

impl fmt::Display for RadosError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RadosError::FromUtf8Error(ref e) => f.write_str(&e.to_string()),
            RadosError::NulError(ref e) => f.write_str(&e.to_string()),
            RadosError::Error(ref e) => f.write_str(&e),
            RadosError::IoError(ref e) => f.write_str(&e.to_string()),
            RadosError::ApiError(ref e) => e.fmt(f),
            RadosError::IntoStringError(ref e) => f.write_str(&e.to_string()),
            RadosError::UuidError(ref e) => f.write_str(&e.to_string()),
            RadosError::ParseBoolError(ref e) => f.write_str(&e.to_string()),
            RadosError::ParseIntError(ref e) => f.write_str(&e.to_string()),
            RadosError::SerdeError(ref e) => f.write_str(&e.to_string()),
            RadosError::MinVersion(ref _min, ref _current_version) => {
                f.write_str("Ceph version is too low")
            }
            RadosError::Parse(ref _input) => f.write_str("An error occurred during parsing"),
        }
    }
}

impl StdError for RadosError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            RadosError::FromUtf8Error(ref e) => e.source(),
            RadosError::NulError(ref e) => e.source(),
            RadosError::Error(ref _e) => None,
            RadosError::IoError(ref e) => e.source(),
            RadosError::ApiError(ref e) => e.source(),
            RadosError::IntoStringError(ref e) => e.source(),
            RadosError::UuidError(ref e) => e.source(),
            RadosError::ParseBoolError(ref e) => e.source(),
            RadosError::ParseIntError(ref e) => e.source(),
            RadosError::SerdeError(ref e) => e.source(),
            RadosError::MinVersion(ref _min, ref _current_version) => None,
            RadosError::Parse(ref _input) => None,
        }
    }
}

impl RadosError {
    /// Create a new RadosError with a String message
    pub fn new(err: String) -> RadosError {
        RadosError::Error(err)
    }
}

impl From<UuidError> for RadosError {
    fn from(err: UuidError) -> RadosError {
        RadosError::UuidError(err)
    }
}

impl From<ParseBoolError> for RadosError {
    fn from(err: ParseBoolError) -> RadosError {
        RadosError::ParseBoolError(err)
    }
}

impl From<ParseIntError> for RadosError {
    fn from(err: ParseIntError) -> RadosError {
        RadosError::ParseIntError(err)
    }
}

impl From<SerdeJsonError> for RadosError {
    fn from(err: SerdeJsonError) -> RadosError {
        RadosError::SerdeError(err)
    }
}

impl From<NulError> for RadosError {
    fn from(err: NulError) -> RadosError {
        RadosError::NulError(err)
    }
}

impl From<FromUtf8Error> for RadosError {
    fn from(err: FromUtf8Error) -> RadosError {
        RadosError::FromUtf8Error(err)
    }
}
impl From<IntoStringError> for RadosError {
    fn from(err: IntoStringError) -> RadosError {
        RadosError::IntoStringError(err)
    }
}
impl From<Error> for RadosError {
    fn from(err: Error) -> RadosError {
        RadosError::IoError(err)
    }
}
impl From<i32> for RadosError {
    fn from(err: i32) -> RadosError {
        RadosError::ApiError(nix::errno::Errno::from_i32(-err))
    }
}
