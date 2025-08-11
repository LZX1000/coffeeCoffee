use anyhow::Result;

use std::{collections::HashMap, fs};

use serde::Deserialize;

use crate::hoverable::Hoverable;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_customer_arrival_wait")]
    pub customer_arrival_wait: usize,
    
    #[serde(default = "default_max_line_size")]
    pub max_line_size: usize,

    #[serde(default = "default_right_side_menu_padding")]
    pub right_side_menu_padding: usize,

    #[serde(default = "default_drinks")]
    pub drinks: HashMap<String, Vec<Hoverable>>,
}

impl Config {
    pub fn new(path: &str) -> Result<Config> {
        let data: String = fs::read_to_string(path)?; // read file as string
        let config: Config = serde_json::from_str(&data)?; // parse JSON
        Ok(config)
    }
}

fn default_customer_arrival_wait() -> usize { 10 }
fn default_max_line_size() -> usize { 10 }
fn default_right_side_menu_padding() -> usize { 1 }
fn default_drinks() -> HashMap<String, Vec<Hoverable>> {
    let mut map = HashMap::new();
    map.insert(
        "0".to_string(),
        vec![Hoverable {
            text: "Coffee".to_string(),
            ..Default::default()
        }]
    );
    map
}