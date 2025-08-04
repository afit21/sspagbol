mod netutils;

use ansi_term::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};
use serde::Deserialize;
use netutils::{ping, web_server_up};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigItem {
    pub ciname: String,
    pub citype: String, //Defines the type of Config Item that this is
    pub cidata: Vec<String> //Undefined data strings

    //CI Data:
    //Hostmachine
    //  [0] - IP address
    //Webserver
    //  [0] - URL
    //  [1] - Port
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub name: String,
    pub desc: String,
    pub cilist: Vec<ConfigItem>
}

impl ConfigItem {
    //Prints the status of a Config Item
    pub fn print_status(self) {
        let ciclone = self.clone();
        let handle = match self.citype.as_str() {
            "Hostmachine" => print_hostmachine_status(ciclone),
            "Webserver" => print_webserver_status(ciclone),
            _ => print_hostmachine_status(ciclone),
        };

        handle.join().expect("Thread panicked");
    }

    //Verifies a CI's data is valid
    pub fn verify_valid_data(&self) -> bool {
        match self.citype.as_str() {
            "Hostmachine" => return verify_hostmachine_data(self),
            "Webserver" => return verify_webserver_data(self),
            _ =>  return verify_hostmachine_data(self),
        };
    }
}

impl Service {
    pub fn print_srv_status(self) {
        let serviceclone = self.clone();
        print_service_status_header(serviceclone.name, serviceclone.desc);
        for ci in serviceclone.cilist {
            if ci.verify_valid_data() {
                ci.print_status();
            } else {
                print!("Invalid data for Configuration Item {}", ci.ciname)
            }
        }
    }
}

pub fn load_services_from_yaml(path: &str) -> Result<Vec<Service>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let services: Vec<Service> = serde_yaml::from_reader(reader)?;
    Ok(services)
}

fn verify_hostmachine_data(ci: &ConfigItem) -> bool {
    //TODO: Implement this
    return true;
}

fn verify_webserver_data(ci: &ConfigItem) -> bool {
    //TODO: Implement this
    return true;
}

//Spinner for loading
fn create_spinner(label: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("        {msg}{spinner}")
            .expect("Failed to set progress bar style"),
    );
    pb.set_message(label.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

//
fn run_status_check<F>(label: &str, task: F) -> thread::JoinHandle<()> where F: FnOnce() -> String + Send + 'static, {
    let pb = create_spinner(label.to_string().as_str());

    thread::spawn(move || {
        let result_msg = task();
        pb.finish_with_message(result_msg);
    })
}

//Prints the header lines when printing a Congig Item Status
fn print_ci_status_header(ciname: String, text: String) {
    println!(
        "    {}",
        Style::new().bold().paint(format!("{} - {}", ciname, text))
    );
}

//Prints the header lines when printing a Service Status
fn print_service_status_header(svcname: String, text: String) {
    let content = format!("{} - {}", svcname, text);
    let styled_content = Style::new().bold().paint(&content).to_string();
    let width = content.chars().count();
    let horizontal_border = format!("+{}+", "-".repeat(width + 2));

    println!(" ");
    println!("{}", horizontal_border);
    println!("| {} |", styled_content);
    println!("{}", horizontal_border);
}

//Appends emoji depending on bool
fn print_bool_result_emoji(text: &str, result: bool) -> String {
    format!("{}: {}", text, if result { "✅" } else { "❌" })
}

//Print Host Machine Status
fn print_hostmachine_status(config: ConfigItem) -> thread::JoinHandle<()> {
    print_ci_status_header(config.ciname, "Host Machine".to_string());

    run_status_check("", move || {
        //Status results
        print_bool_result_emoji("Can ping", ping(config.cidata[0].as_str()))
    })
}

//Print Web Server Status
fn print_webserver_status(config: ConfigItem) -> thread::JoinHandle<()> {
    print_ci_status_header(config.ciname, "Web Server".to_string());

    run_status_check("", move || {
        //Status results
        let port : u16 = config.cidata[1].parse().expect("Invalid number");
        //TODO: add error handling for above
        let addr = config.cidata[0].as_str();
        print_bool_result_emoji("Web Server Up", web_server_up(addr, port))
    })
}