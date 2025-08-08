use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};
use tokio::task;
use crossterm::event::{self, Event, KeyCode};


#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    let counting = Arc::new(Mutex::new(true));

    let counter_clone = Arc::clone(&counter);
    let counting_clone = Arc::clone(&counting);

    task::spawn(async move {
        while *counting_clone.lock().unwrap() {
            {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            }
            sleep(Duration::from_secs(1)).await
        }
    });

    {
        let mut pressed = false;

        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Enter => {
                            pressed = !pressed;
                            if !pressed {
                                let value = counter.lock().unwrap();
                                println!("Counter: {}", value);
                            }
                        }
                        KeyCode::Esc => {
                            *counting.lock().unwrap() = false;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
