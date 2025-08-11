mod helpers;
mod gameplay;
mod hoverable;
mod customer;
mod player;
mod config;
mod events;

use anyhow::Result;

const CONFIG_PATH: &str = "config.json";

fn main() -> Result <()> {
    helpers::alternate_screen_wrapper(|| gameplay::main(CONFIG_PATH))?;
    Ok(())
}
