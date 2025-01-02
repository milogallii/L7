use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct PolicyParser {
    policy: HashMap<String, Component>,
}

#[derive(Debug, Deserialize)]
struct Component {
    name: String,
    sends: Vec<String>,
    receives: Vec<String>,
}

impl PolicyParser {
    pub fn new(policy_file_path: String) -> Self {
        let toml_content =
            fs::read_to_string(policy_file_path).expect("Failed to read config.toml");

        let policy_parser: PolicyParser =
            toml::from_str(&toml_content).expect("Failed to parse config.toml");

        policy_parser
    }

    pub fn show_policy(&self) {
        self.policy.iter().for_each(|(_, field)| {
            println!("  ----------------");
            println!("  Name: {}", field.name);
            println!("  Sends: {:?}", field.sends);
            println!("  Receives: {:?}", field.receives);
            println!("  ----------------");
        });
    }
}
