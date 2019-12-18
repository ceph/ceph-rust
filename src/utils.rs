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

use std::io::Result;

use std::process::{Command, Output};

/// run_cli - pass in a String of a normal command line
///
/// The function will split the options into words to supply to the low_level
/// std::process::Command
/// which returns Result<(Output)>
/// # Example
///
/// ```
/// use ceph::utils::run_cli;
/// run_cli("ps aux");
/// ```

// NOTE: Add Into so a "" can also be passed in...
pub fn run_cli(cmd_line: &str) -> Result<Output> {
    let output = Command::new("sh").arg("-c").arg(cmd_line).output()?;
    Ok(output)
}
