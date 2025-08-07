mod services;
use services::{load_services_from_yaml, collect_all_services_in_parallel};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

// Splash title
fn print_splash() {
    let banner = r#"
   _____ _____ _____        _____ ____   ____  _      
  / ____/ ____|  __ \ /\   / ____|  _ \ / __ \| |     
 | (___| (___ | |__) /  \ | |  __| |_) | |  | | |     
  \___ \\___ \|  ___/ /\ \| | |_ |  _ <| |  | | |     
  ____) |___) | |  / ____ \ |__| | |_) | |__| | |____ 
 |_____/_____/|_| /_/    \_\_____|____/ \____/|______|
                                                     
"#;
    println!("{}", banner);
}

// Spinner function
fn start_spinner(message: &str) -> (Arc<AtomicBool>, thread::JoinHandle<()>) {
    let running = Arc::new(AtomicBool::new(true));
    let spinner_running = Arc::clone(&running);
    let msg = message.to_string();

    let handle = thread::spawn(move || {
        let spinner_chars = ['⠋', '⠙', '⠚', '⠞', '⠖', '⠦', '⠴', '⠲', '⠳', '⠓'];
        let mut i = 0;
        while spinner_running.load(Ordering::SeqCst) {
            print!("\r{} {}", msg, spinner_chars[i % spinner_chars.len()]);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
            i += 1;
        }

        // Clear spinner line
        let clear_line = " ".repeat(msg.len() + 2);
        print!("\r{}\r", clear_line); // overwrite and return carriage
        io::stdout().flush().unwrap();
    });

    (running, handle)
}

fn main() {
    print_splash();
    let config_path = "/etc/sspagbol/services.yaml";
    let services = match load_services_from_yaml(config_path) {
        Ok(srv) => srv,
        Err(e) => {
            eprintln!("Error loading services: {}", e);
            return;
        }
    };

    // Start spinner
    let (spinner_flag, spinner_handle) = start_spinner("Running Status Checks...");

    // This is where the magic happens
    let output = collect_all_services_in_parallel(&services);

    // Stop spinner
    spinner_flag.store(false, Ordering::SeqCst);
    spinner_handle.join().unwrap();

    // Print final output
    println!("{}", output);
}
