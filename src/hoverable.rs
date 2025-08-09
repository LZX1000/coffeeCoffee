use serde::Deserialize;

const REVERSE: &str = "\x1B[7m";
const RESET: &str = "\x1B[0m";

#[derive(Debug, Deserialize)]
pub struct Hoverable {
    #[serde(default = "default_text")]
    text: String,

    #[serde(default = "default_font")]
    font: String,

    #[serde(default = "default_hovered")]
    hovered: bool,
}

fn default_text() -> String { "".to_string() }
fn default_font() -> String { RESET.to_string() }
fn default_hovered() -> bool { false } 

impl From<Hoverable> for String {
    fn from(obj: Hoverable) -> Self {
        if obj.hovered {
            format!("{}{}{}{}", obj.font, REVERSE, obj.text, RESET)
        } else {
            format!("{}{}{}", obj.font, obj.text, RESET)
        }
        .to_string() 
    }
}