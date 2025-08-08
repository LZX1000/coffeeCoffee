use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};
use tokio::task;
use serde::Deserialize;
use std::fs;
use std::io::{stdout, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    // style::{Print},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};


#[derive(Debug, Deserialize)]
struct Config {
    customer_arrival_rate: u64,
    drinks: Vec<String>,
}


fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?; // read file as string
    let config: Config = serde_json::from_str(&data)?; // parse JSON
    Ok(config)
}


#[tokio::main]
async fn main() -> Result <(), Box<dyn std::error::Error>> {
    // Enter Alternate Screen
    let mut main_stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(main_stdout, EnterAlternateScreen, cursor::Hide)?;

    // Setup Arcs
    let cfg = {
        let cfg_raw = load_config("config.json")
        .map_err(|e| {
            eprintln!("Failed to load config: {}", e);
            e
        })?;
        
        Arc::new(cfg_raw)
    };
    let running = Arc::new(Mutex::new(true));
    let stdout = Arc::new(Mutex::new(stdout()));

    // Rendering
    let running_render = Arc::clone(&running);
    let stdout_render = Arc::clone(&stdout);
    let cfg_render = cfg.clone();
    let render_handle = task::spawn(async move {
        let mut last_render = String::new();

        while *running_render.lock().unwrap() {
            {
                let drinks_formatted = cfg_render
                    .drinks
                    .iter()
                    .map(|drink| format!("  {}", drink))
                    .collect::<Vec<_>>()
                    .join("\n");

                let current_render = format!(
                    "Arrival Rate: {}\nDrinks:\n{}\n\nPress 'esc' to quit.",
                    cfg_render.customer_arrival_rate,
                    drinks_formatted
                );

                if current_render != last_render {
                    let mut out = stdout_render.lock().unwrap();
                    execute!(*out, cursor::MoveTo(0, 0), Clear(ClearType::FromCursorDown)).ok();
                    writeln!(*out, "{}", current_render).ok();
                    out.flush().ok();
                    last_render = current_render;
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
    });

    // Controlling
    let running_input = Arc::clone(&running);
    let input_handle = task::spawn(async move {
        while *running_input.lock().unwrap() {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Esc => {
                            *running.lock().unwrap() = false;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let (input_res, render_res) = tokio::join!(
        input_handle,
        render_handle,
    );

    input_res.unwrap();
    render_res.unwrap();

    // Restore terminal
    execute!(main_stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
