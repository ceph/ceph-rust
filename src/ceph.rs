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

#![cfg(target_os = "linux")]
use std::slice;
use std::io::Error;

use rados::*;

/// Returns back a collection of Rados Pools
///
/// pool_buffer should be allocated with:
/// ```
/// let pool_buffer: Vec<u8> = Vec::with_capacity(<whatever size>);
/// ```
/// buf_size should be the value used with_capacity
///
/// Returns Ok(Vec<String>) - A list of Strings of the pool names.
///
#[allow(unused_variables)]
pub fn rados_pools(cluster: rados_t)
                   -> Result<Vec<String>, Error> {
  let mut pools: Vec<String> = Vec::new();
  let pool_slice: &[u8];
  let buf_size: usize = 500;
  let mut pool_buffer: Vec<u8> = Vec::with_capacity(buf_size);

  unsafe {
    // Don't need len but did it anyway
    let len = rados_pool_list(cluster,
                              pool_buffer.as_mut_ptr() as *mut i8,
                              buf_size);

    pool_slice = slice::from_raw_parts(pool_buffer.as_mut_ptr(), buf_size);

    // NOTE: Pushed for time so did this. If anyone knows of a better way then please issue a PR.
    let mut new: bool =  true;
    let mut new_word_slice: Vec<u8> = Vec::with_capacity(80);  // More than 
    let mut s: String;

    for p in pool_slice.chunks(1) {
      if p[0] == b'\0' {
        if new {
          break;
        }
        new = true;
        s = String::from_utf8(new_word_slice.clone()).unwrap();
        pools.push(s.clone());
        new_word_slice.clear();
        continue;
      }

      new_word_slice.push(p[0]);
      new = false;
    }
  }

  Ok(pools)
}
