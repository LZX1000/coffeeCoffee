use serde::{self, Deserialize};
use std::fmt;

const REVERSE: &str = "\x1B[7m";
const RESET: &str = "\x1B[0m";

#[derive(Debug)]
pub struct Hoverable {
    pub text: String,
    pub font: String,
    pub hovered: bool,
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
                hovered: full.hovered
            }
        }
    }
}

fn default_text() -> String { "".to_string() }
fn default_font() -> String { RESET.to_string() }
fn default_hovered() -> bool { false } 

// impl From<Hoverable> for String {
//     fn from(obj: Hoverable) -> Self {
//         if obj.hovered {
//             format!("{}{}{}{}", obj.font, REVERSE, obj.text, RESET)
//         } else {
//             format!("{}{}{}", obj.font, obj.text, RESET)
//         }
//     }
// }

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
        }
    }
}

impl <'de> Deserialize<'de> for Hoverable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let text = String::deserialize(deserializer)?;
        Ok(Hoverable {
            text,
            ..Default::default()
        })
    }
}