use serde::{self, Deserialize};
use serde_json::Value;
use std::{collections::HashMap, fmt};

const REVERSE: &str = "\x1B[7m";
const RESET: &str = "\x1B[0m";

#[derive(Debug, Clone, PartialEq)]
pub struct Hoverable {
    pub text: String,
    pub font: String,
    pub hovered: bool,
    pub data: Option<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum HoverableDef {
    Text(String),
    Full(HoverableFull),
}

#[derive(Debug, Deserialize)]
struct HoverableFull {
    #[serde(default = "default_text")]
    text: String,

    #[serde(default = "default_font")]
    font: String,

    #[serde(default = "default_hovered")]
    hovered: bool,

    #[serde(default)]
    data: Option<HashMap<String, Value>>,
}

impl From<HoverableDef> for Hoverable {
    fn from(hd: HoverableDef) -> Self {
        match hd {
            HoverableDef::Text(text) => Hoverable {
                text,
                ..Default::default()
            },
            HoverableDef::Full(full) => Hoverable {
                text: full.text,
                font: full.font,
                hovered: full.hovered,
                data: full.data,
            }
        }
    }
}

fn default_text() -> String { "".to_string() }
fn default_font() -> String { RESET.to_string() }
fn default_hovered() -> bool { false } 

impl fmt::Display for Hoverable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.hovered {
            write!(f, "{}{}{}{}", self.font, REVERSE, self.text, RESET)
        } else {
            write!(f, "{}{}{}", self.font, self.text, RESET)
        }
    }
}

impl Default for Hoverable {
    fn default() -> Self {
        Self {
            text: "".to_string(),
            font: RESET.to_string(),
            hovered: false,
            data: None,
        }
    }
}

impl <'de> Deserialize<'de> for Hoverable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
    {
        let helper = HoverableDef::deserialize(deserializer)?;
        Ok(helper.into())
    }
}