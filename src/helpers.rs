use std::{io::stdout, sync::{Arc, Condvar, Mutex}, time::{Duration, Instant}};

use anyhow::Result;

use crossterm::{cursor, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};

pub fn alternate_screen_wrapper<F, R>(func: F) -> Result<R> 
    where F: FnOnce() -> Result<R>,
{
    let mut main_stdout = stdout();

    // Setup
    terminal::enable_raw_mode()?;
    execute!(main_stdout, EnterAlternateScreen, cursor::Hide)?;

    // Run
    let result = func();

    // Cleanup
    let mut cleanup = || -> Result<()> {
        execute!(main_stdout, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    };

    match result {
        Ok(val) => {
            cleanup()?;
            Ok(val)
        }
        Err(e) => {
            cleanup()?;
            Err(e)
        }
    }
}

pub fn cancellable_sleep(condvar_pair: &(Arc<Mutex<bool>>, Arc<Condvar>), max_wait:Duration) {
    let (lock, cvar) = condvar_pair;
    let mut flag = lock.lock().unwrap();
    let start = Instant::now();

    while !*flag {
        let elapsed = start.elapsed();
        if elapsed >= max_wait {
            break;
        }
        let wait_duration = max_wait - elapsed;
        let (new_flag, _) = cvar.wait_timeout(flag, wait_duration).unwrap();
        flag = new_flag;
    }
    *flag = false;
}