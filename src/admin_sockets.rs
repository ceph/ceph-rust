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

#![cfg(unix)]

use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{RadosError, RadosResult};
use std::io::{Cursor, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::str;

/// This is a helper function that builds a raw command from the actual
/// command. You just pass
/// in a command like "help". The returned `String` will be a JSON String.
pub fn admin_socket_command(cmd: &str, socket: &str) -> RadosResult<String> {
    let raw_cmd = json!({
        "prefix": cmd,
    });
    admin_socket_raw_command(&raw_cmd.to_string(), socket)
}

/// This function supports a raw command in the format of something like:
/// `{"prefix": "help"}`.
/// The returned `String` will be a JSON String.
#[allow(unused_variables)]
pub fn admin_socket_raw_command(cmd: &str, socket: &str) -> RadosResult<String> {
    let mut buffer = vec![0; 4]; // Should return 4 bytes with size or indicator.
    let cmd = &format!("{}\0", cmd); // Terminator so don't add one to commands.

    let mut stream = UnixStream::connect(socket)?;
    let wb = stream.write(cmd.as_bytes())?;
    let ret_val = stream.read(&mut buffer)?;
    if ret_val < 4 {
        stream.shutdown(Shutdown::Both)?;
        return Err(RadosError::new(
            "Admin socket: Invalid command or socket did not return any data".to_string(),
        ));
    }
    // The first 4 bytes are Big Endian unsigned int
    let mut rdr = Cursor::new(buffer);
    let len = rdr.read_u32::<BigEndian>()?;
    let mut output_buffer = vec![0; len as usize];
    stream.read_exact(&mut output_buffer)?;
    stream.shutdown(Shutdown::Both)?;

    Ok(String::from_utf8_lossy(&output_buffer).into_owned())
}
