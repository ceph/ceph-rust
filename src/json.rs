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

use std::str;

use serde_json;

use crate::JsonData;
// use JsonValue;

/// First json call that takes a JSON formatted string and converts it to
/// JsonData object that can then be traversed using `json_find` via the key
/// path.
pub fn json_data(json_str: &str) -> Option<JsonData> {
    match serde_json::from_str(json_str) {
        Ok(json_data) => Some(json_data),
        Err(_) => None,
    }
}

/// Looks for the parent object first and then the 'child' object. If the
/// parent object is None then it only looks for the 'child' object. The parent
/// object is used for situations where there may be 'child' objects with the
/// same name.
pub fn json_find(json_data: JsonData, keys: &[&str]) -> Option<JsonData> {
    let mut value = json_data;
    for key in keys {
        match value.get(key) {
            Some(v) => value = v.clone(),
            None => return None,
        }
    }

    Some(value)
}

/// More specific String cast of an individual JsonData object.
pub fn json_as_string(json_data: &JsonData) -> String {
    json_data.to_string()
}
