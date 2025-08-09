mod customer;
mod player;

use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// use std::thread::current;
use tokio::task;
use serde::Deserialize;
use std::fs;
use std::io::{stdout, Write};
use futures::future::join_all;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    // style::{Print},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};


#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "default_customer_arrival_wait")]
    customer_arrival_wait: usize,
    
    #[serde(default = "default_max_line_size")]
    max_line_size: usize,

    #[serde(default = "default_right_side_menu_padding")]
    right_side_menu_padding: usize,

    #[serde(default = "default_drinks")]
    drinks: HashMap<String, Vec<String>>,
}

fn default_customer_arrival_wait() -> usize { 10 }
fn default_max_line_size() -> usize { 10 }
fn default_right_side_menu_padding() -> usize { 1 }
fn default_drinks() -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    map.insert("0".to_string(), vec!["Coffee".to_string()]);
    map
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let data: String = fs::read_to_string(path)?; // read file as string
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
    let customers = Arc::new(Mutex::new(Vec::<customer::Customer>::new()));
    let mut player = Arc::new(player::Player::new());
    let mut handles = Vec::new();

    // Rendering
    let running_render = Arc::clone(&running);
    let stdout_render = Arc::clone(&stdout);
    let cfg_render = cfg.clone();
    let player_render = Arc::clone(&player);
    let customers_render = Arc::clone(&customers);

    let render_handle = task::spawn(async move {
        let max_drinks_width = {   
            cfg_render
                .drinks
                .values()
                .flat_map(|drinks| {
                    let mut lines = vec!["Drinks:".to_string()];
                    lines.extend(drinks.iter().map(|d| format!("  {}", d)));
                    lines
                })
                .map(|line| line.len())
                .max()
                .unwrap_or(0)
        };
        let render_widths = vec![cfg_render.right_side_menu_padding + max_drinks_width];
        let mut last_render: Vec<String> = vec!["".to_string(); 2]; // Number of menus
        loop {
            if !*running_render.lock().unwrap() {
                break;
            }
            
            {
                let mut current_render: Vec<String> = Vec::new();
                // let level_key = player_render.level().to_string();
                
                let drinks_menu: String = format!(
                    "Drinks:\n{}",
                    cfg_render
                        .drinks
                        .get(&player_render.level().to_string())
                        .map(|drinks| {
                            drinks
                                .iter()
                                .map(|drink| format!("  {}", drink))
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_else(|| "  No drinks available.".to_string())
                );
                current_render.push(drinks_menu);

                let customers_menu: String = {
                    let len = customers_render.lock().unwrap().len();

                    (0..cfg_render.max_line_size)
                        .map(|i| {
                            if i < len {
                                "|"
                            } else {
                                "-"
                            }
                            .to_string()
                        })
                        .collect::<String>()
                };
                current_render.push(customers_menu);

                let mut out = stdout_render.lock().unwrap();
                if current_render != last_render {
                    let mut left_gap: usize = 1;
                    for i in 0..current_render.len() {
                        let width = match render_widths.get(i) {
                            Some(value) => value,
                            None => &current_render[i]
                                .lines()
                                .map(|line| line.len())
                                .max()
                                .unwrap_or(0)
                        };
                        if current_render[i] != last_render[i] {
                            write!(
                                *out,
                                "\x1B[1;{}H{:<width$}",
                                left_gap, current_render[i]
                            ).ok();
                        }
                        left_gap += width;
                    };
                    out.flush().ok();
                    last_render = current_render;
                }
            }

            sleep(Duration::from_millis(100)).await;
        }
    });
    handles.push(render_handle);

    // Controlling
    let running_input = Arc::clone(&running);

    let input_handle = task::spawn(async move {
        loop {
            if !*running_input.lock().unwrap() {
                break;
            }

            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Esc => {
                            *running_input.lock().unwrap() = false;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    handles.push(input_handle);

    let doing_customer_spawner = Arc::new(Mutex::new(true));
    let customer_spwaner_doing_customer_spawner = Arc::clone(&doing_customer_spawner);

    let running_customer_spawner = Arc::clone(&running);
    let cfg_customer_spawner = Arc::clone(&cfg);
    let customers_customer_spawner = Arc::clone(&customers);

    let customer_spawner_handle = task::spawn(async move {
        loop {
            if !*running_customer_spawner.lock().unwrap() {
                break;
            }
            if !*customer_spwaner_doing_customer_spawner.lock().unwrap() {
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let sleep_duration = cfg_customer_spawner.customer_arrival_wait;
            sleep(Duration::from_secs(sleep_duration.try_into().unwrap())).await;
            {
                let mut customers_customer_spawner_locked = customers_customer_spawner.lock().unwrap();
                if cfg_customer_spawner.max_line_size > customers_customer_spawner_locked.len() {
                    (customers_customer_spawner_locked).push(customer::Customer::new())
                } else {
                    {
                        *running_customer_spawner.lock().unwrap() = false;
                    }

                    println!("Game Over! Too many customers in line.");
                    break;
                }
            }
        }
    });
    handles.push(customer_spawner_handle);

    join_all(handles).await;

    println!("Press `Esc` to exit.\nPress `Enter` to continue.");

    // Restore terminal
    execute!(main_stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
