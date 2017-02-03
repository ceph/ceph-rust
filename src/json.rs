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

use serde_json::*;

use JsonData;
use JsonValue;

pub fn json_data(json_str: &str) -> Option<JsonData> {
    match from_str(json_str) {
        Ok(json_data) => {
            Some(json_data)
        },
        Err(e) => {
            println!("{}", e);
            None
        }
    }

}

pub fn json_find(json_data: &JsonData, object: &str) -> Option<JsonValue> {
    let json_value: Option<JsonValue> = Some(json_data.as_object().unwrap().get(object)
                .and_then(|value| Some(value.to_string()))
                .unwrap_or_else(|| {
                    "".to_string()
                }));

    json_value
}
