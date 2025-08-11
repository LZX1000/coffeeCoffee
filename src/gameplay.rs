use anyhow::Result;

use crossbeam::channel::{unbounded, Receiver, Sender};

use crossterm::event::{self, Event, KeyCode};

use std::{io::{stdout, Write}, sync::{atomic::{AtomicBool, Ordering::SeqCst}, Arc, Condvar, Mutex}, thread, time::Duration};

use crate::{config::Config, customer::Customer, events::Event as CodeEvent, helpers::cancellable_sleep, hoverable::Hoverable, player::Player};

pub fn main(cfg_path: &str) -> Result<()> {
    // Setup Arcs
    let cfg: Config = Config::new(cfg_path)?;

    // let mut selected_drink: Hoverable = Hoverable::default();
    // let mut selected_menu: Vec<Hoverable> = Vec::<Hoverable>::new();

    let running = Arc::new(AtomicBool::new(true));
    let doing_customer_spawner = Arc::new(AtomicBool::new(true));
    let customers = Arc::new(Mutex::new(Vec::<Customer>::new()));
    let player = Arc::new(Mutex::new(Player::new()));
    let buttons = Arc::new(Mutex::new(vec![vec![]]));
    let stdout = Arc::new(Mutex::new(stdout()));
    let sleep_flag = Arc::new(Mutex::new(false));
    let sleep_cvar = Arc::new(Condvar::new());
    let condvar_pair = (Arc::clone(&sleep_flag), Arc::clone(&sleep_cvar));

    let (event_tx, event_rx): (Sender<CodeEvent>, Receiver<CodeEvent>) = unbounded(); // To use MPMC

    // Rendering
    {
        let running = Arc::clone(&running);
        let customers = Arc::clone(&customers);
        let player = Arc::clone(&player);
        let buttons = Arc::clone(&buttons);
        let stdout = Arc::clone(&stdout);
        let condvar_pair = (
            Arc::clone(&condvar_pair.0),
            Arc::clone(&condvar_pair.1),
        );
        let cfg = cfg.clone();

        thread::spawn(move || {
            let max_drinks_width = {   
                cfg
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
            let render_widths = vec![cfg.right_side_menu_padding + max_drinks_width];
            let mut last_render: Vec<String> = vec!["".to_string(); 2]; // Number of menus
            while running.load(SeqCst) {
                let mut current_render: Vec<String> = Vec::new();

                let player_guard = player.lock().unwrap();
                let level_key = player_guard.level().to_string();
                drop(player_guard);

                let drinks_menu: String = format!(
                    "Drinks:\n{}",
                    cfg
                        .drinks
                        .get(&level_key)
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

                let customers_guard = customers.lock().unwrap();
                let customers_menu: String = {
                    let len = customers_guard.len();

                    (0..cfg.max_line_size)
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
                drop(customers_guard);
                current_render.push(customers_menu);

                if current_render != last_render {
                    let mut out = stdout.lock().unwrap();
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

            cancellable_sleep(&condvar_pair, Duration::from_millis(100));
        });
    }

    // --- Buttons ---
{
    let running = Arc::clone(&running);
    let player = Arc::clone(&player);
    let buttons = Arc::clone(&buttons);
    let cfg = cfg.clone();

    thread::spawn(move || {
        while running.load(SeqCst) {
            let level_key = {
                let player_guard = player.lock().unwrap();
                player_guard.level().to_string()
            };

            let drinks: Vec<Hoverable> = cfg
                .drinks
                .get(&level_key)
                .cloned()
                .unwrap_or_default();

            let new_buttons: Vec<Vec<Hoverable>> = vec![
                drinks
            ];

            {
                let mut buttons_guard = buttons.lock().unwrap();
                if new_buttons != *buttons_guard {
                    *buttons_guard = new_buttons;
                }
            }
            cancellable_sleep(&condvar_pair, Duration::from_secs(1));
        }
    });
}

    // Input Handler
    {
        let running = Arc::clone(&running);
        let doing_customer_spawner = Arc::clone(&doing_customer_spawner);
        let event_tx = event_tx.clone();
        let event_rx = event_rx.clone();
        
        thread::spawn(move || {
            // let mut running_controller = &mut running;

            while running.load(SeqCst) {
                if event::poll(Duration::from_millis(10)).unwrap() {
                    // Key Events
                    if let Event::Key(key_event) = event::read().unwrap() {
                        match key_event.code {
                            KeyCode::Esc => {
                                let _ = event_tx.send(CodeEvent::Quit);
                                break;
                            }
                            _ => {}
                        }
                    }
                    // Code Events
                    match event_rx.recv() {
                        Ok(CodeEvent::Quit) => {
                            running.store(false, SeqCst);
                        }
                        Ok(CodeEvent::StopCustomerSpawning) => {
                            doing_customer_spawner.store(false, SeqCst)
                        }
                        Ok(CodeEvent::SpawnCustomer) => {
                            // Do stuff
                        }
                        _ => { }
                    }
                }
            }
        });
    }

    // --- Event Handler ---
    {
        let running = Arc::clone(&running);
        let doing_customer_spawner = Arc::clone(&doing_customer_spawner);
        let customers = Arc::clone(&customers);
        let event_rx = event_rx.clone();

        thread::spawn(move || {
            while running.load(SeqCst) {
                match event_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(event) => match event {
                        CodeEvent::Quit => {
                            running.store(false, SeqCst);
                            {
                                let mut flag = sleep_flag.lock().unwrap();
                                *flag = true;
                                sleep_cvar.notify_all();
                            }
                        }
                        CodeEvent::StopCustomerSpawning => {
                            doing_customer_spawner.store(false, SeqCst);
                        }
                        CodeEvent::SpawnCustomer => {
                            let mut customers_guard = customers.lock().unwrap();
                            if customers_guard.len() < cfg.max_line_size {
                                customers_guard.push(Customer::new());
                            } else {
                                running.store(false, SeqCst);
                            }
                        }
                        _ => { }
                    },
                    Err(_) => { /* Timeout - loop */ }
                }
            }
        });
    }

    // --- Customer Spawning ---
    {
        let running = Arc::clone(&running);
        let doing_customer_spawner = Arc::clone(&doing_customer_spawner);
        let event_tx = event_tx.clone();
        let cfg = cfg.clone();

        thread::spawn(move || {
            while running.load(SeqCst) {
                if !doing_customer_spawner.load(SeqCst) {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }

                thread::sleep(Duration::from_secs(cfg.customer_arrival_wait as u64));
                if !running.load(SeqCst) {
                    break;
                }

                let _ = event_tx.send(CodeEvent::SpawnCustomer);
            }
        });
    }

    while running.load(SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}