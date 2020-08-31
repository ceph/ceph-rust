use std::collections::HashMap;

use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_a_mon_command() {
        let command = MonCommand::new()
            .with_prefix("osd set")
            .with("key", "osdout");

        let actual: HashMap<String, String> = serde_json::from_str(&command.as_json()).unwrap();
        let expected: HashMap<String, String> =
            serde_json::from_str(r#"{"prefix":"osd set","format":"json","key":"osdout"}"#).unwrap();

        assert_eq!(expected, actual);
    }
}

pub struct MonCommand<'a> {
    map: HashMap<&'a str, &'a str>,
}

impl<'a> Default for MonCommand<'a> {
    fn default() -> Self {
        MonCommand {
            map: {
                let mut map = HashMap::new();
                map.insert("format", "json");
                map
            },
        }
    }
}

impl<'a> MonCommand<'a> {
    pub fn new() -> MonCommand<'a> {
        MonCommand::default()
    }

    pub fn with_format(self, format: &'a str) -> MonCommand<'a> {
        self.with("format", format)
    }

    pub fn with_name(self, name: &'a str) -> MonCommand<'a> {
        self.with("name", name)
    }

    pub fn with_prefix(self, prefix: &'a str) -> MonCommand<'a> {
        self.with("prefix", prefix)
    }

    pub fn with(mut self, name: &'a str, value: &'a str) -> MonCommand<'a> {
        self.map.insert(name, value);
        self
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(&self.map).unwrap()
    }
}
