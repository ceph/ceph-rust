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


use serde_json::error::Error as SerdeJsonError;
use std::{fmt, str};
use std::error::Error as StdError;
use std::ffi::{IntoStringError, NulError};
use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use uuid::ParseError;

/// Custom error handling for the library
#[derive(Debug)]
pub enum RadosError {
    FromUtf8Error(FromUtf8Error),
    NulError(NulError),
    Error(String),
    IoError(Error),
    IntoStringError(IntoStringError),
    ParseIntError(ParseIntError),
    ParseError(ParseError),
    SerdeError(SerdeJsonError),
}

pub type RadosResult<T> = Result<T, RadosError>;

impl fmt::Display for RadosError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for RadosError {
    fn description(&self) -> &str {
        match *self {
            RadosError::FromUtf8Error(ref e) => e.description(),
            RadosError::NulError(ref e) => e.description(),
            RadosError::Error(ref e) => &e,
            RadosError::IoError(ref e) => e.description(),
            RadosError::IntoStringError(ref e) => e.description(),
            RadosError::ParseError(ref e) => e.description(),
            RadosError::ParseIntError(ref e) => e.description(),
            RadosError::SerdeError(ref e) => e.description(),
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            RadosError::FromUtf8Error(ref e) => e.cause(),
            RadosError::NulError(ref e) => e.cause(),
            RadosError::Error(_) => None,
            RadosError::IoError(ref e) => e.cause(),
            RadosError::IntoStringError(ref e) => e.cause(),
            RadosError::ParseError(ref e) => e.cause(),
            RadosError::ParseIntError(ref e) => e.cause(),
            RadosError::SerdeError(ref e) => e.cause(),
        }
    }
}

impl RadosError {
    /// Create a new RadosError with a String message
    pub fn new(err: String) -> RadosError {
        RadosError::Error(err)
    }

    /// Convert a RadosError into a String representation.
    pub fn to_string(&self) -> String {
        match *self {
            RadosError::FromUtf8Error(ref err) => err.utf8_error().to_string(),
            RadosError::NulError(ref err) => err.description().to_string(),
            RadosError::Error(ref err) => err.to_string(),
            RadosError::IoError(ref err) => err.description().to_string(),
            RadosError::IntoStringError(ref err) => err.description().to_string(),
            RadosError::ParseError(_) => "Uuid parse error".to_string(),
            RadosError::ParseIntError(ref err) => err.description().to_string(),
            RadosError::SerdeError(ref err) => err.description().to_string(),
        }
    }
}

impl From<ParseError> for RadosError {
    fn from(err: ParseError) -> RadosError {
        RadosError::ParseError(err)
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
