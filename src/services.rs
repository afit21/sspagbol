mod netutils;

use ansi_term::Style;
use serde::Deserialize;
use netutils::{ping, web_server_up, ssh_server_up, dns_server_up};
use std::fs::File;
use std::io::BufReader;
use crossbeam_channel::unbounded;
use std::sync::Mutex;

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
    //SSH Server
    //  [0] - IP Address or Hostname
    //  [1] - Port
    //DNS Server
    //  [0] - IP Address
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub name: String,
    pub desc: String,
    pub cilist: Vec<ConfigItem>
}

impl ConfigItem {
    //Verifies a CI's data is valid
    pub fn verify_valid_data(&self) -> bool {
        return true; // Placeholder for actual validation logic
    }
}

impl Service {
    pub fn collect_srv_status_lines(&self) -> Vec<String> {
        let mut lines = vec![];

        lines.push(" ".to_string());

        // Build header as string lines (with ansi)
        let content = format!("{} - {}", self.name, self.desc);
        let styled_content = Style::new().bold().paint(&content).to_string();
        let width = content.chars().count();
        let horizontal_border = format!("+{}+", "-".repeat(width + 2));
        lines.push(horizontal_border.clone());
        lines.push(format!("| {} |", styled_content));
        lines.push(horizontal_border);

        // Channel to collect per-config item results
        let (sender, receiver) = unbounded();

        crossbeam::thread::scope(|s| {
            for ci in &self.cilist {
                if !ci.verify_valid_data() {
                    sender.send(format!("Invalid data for Configuration Item {}", ci.ciname)).unwrap();
                    continue;
                }
                let sender = sender.clone();
                let ci = ci.clone();
                s.spawn(move |_| {
                    //TODO: Add support for the following services:
                    //SMB
                    //FTP
                    //SMTP
                    let result = match ci.citype.as_str() {
                        //Register Type Print Functions
                        "Hostmachine" => hostmachine_status(ci),
                        "Webserver" => webserver_status(ci),
                        "SSHServer" => ssh_status(ci),
                        "DNSServer" => dns_status(ci),
                        _ => format!("{} - Unknown Type", ci.ciname),
                    };
                    sender.send(result).unwrap();
                });
            }
            drop(sender);
        }).unwrap();

        // Collect all results in the order they come (unordered by nature)
        let mut results: Vec<String> = receiver.iter().collect();
        // To do this, create a map of ciname to index to order results
        let mut index_map = std::collections::HashMap::new();
        for (idx, ci) in self.cilist.iter().enumerate() {
            index_map.insert(ci.ciname.clone(), idx);
        }

        // Sort by index of ciname in cilist
        results.sort_by_key(|res| {
            // Extract the first line to find the ciname: it's before the first " - "
            let first_line = res.lines().next().unwrap_or("");
            let name_part = first_line.split(" - ").next().unwrap_or("");
            *index_map.get(name_part).unwrap_or(&usize::MAX)
        });

        // Append results to lines
        lines.extend(results);

        lines
    }
}

//Will print all services in parallel
pub fn collect_all_services_in_parallel(services: &[Service]) -> String {
    let results: Vec<Mutex<Option<Vec<String>>>> =
        (0..services.len()).map(|_| Mutex::new(None)).collect();

    crossbeam::thread::scope(|s| {
        for (i, service) in services.iter().enumerate() {
            let service = service.clone();
            let slot = &results[i];
            s.spawn(move |_| {
                let lines = service.collect_srv_status_lines();
                *slot.lock().unwrap() = Some(lines);
            });
        }
    }).unwrap();

    // Build final output string
    let mut output = String::new();

    for slot in results.iter() {
        if let Some(lines) = slot.lock().unwrap().as_ref() {
            for line in lines {
                output.push_str(line);
                output.push('\n');
            }
        }
    }

    output
}

//Loads services from a YAML file & Returns a vector of Service structs
pub fn load_services_from_yaml(path: &str) -> Result<Vec<Service>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let services: Vec<Service> = serde_yaml::from_reader(reader)?;
    Ok(services)
}

//Appends emoji depending on bool
fn print_bool_result_emoji(text: &str, result: bool) -> String {

    format!("{}: {}", text, if result { "✅" } else { "❌" })
}

//Host Machine Status
fn hostmachine_status(ci: ConfigItem) -> String {
    let status = print_bool_result_emoji("Can ping", ping(&ci.cidata[0]));
    return format!("{} - Host Machine\n        {}", ci.ciname, status);
}

//Web Server Status
fn webserver_status(ci: ConfigItem) -> String {
    let port: u16 = ci.cidata.get(1).and_then(|p| p.parse().ok()).unwrap_or(80);
    let status = print_bool_result_emoji("Web Server Up", web_server_up(&ci.cidata[0], port));
    return format!("{} - Web Server\n        {}", ci.ciname, status);
}

//SSH Server Status
fn ssh_status(ci: ConfigItem) -> String {
    let port: u16 = ci.cidata.get(1).and_then(|p| p.parse().ok()).unwrap_or(22);
    let status = ssh_server_up(&ci.cidata[0], port);
    return format!("{} - SSH Server\n        SSH Server Up: {}", ci.ciname, status);
}

//DNS Server Status
fn dns_status(ci: ConfigItem) -> String {
    let status = dns_server_up(&ci.cidata[0], "google.com");
    return format!("{} - DNS Server\n        DNS Server Up: {}", ci.ciname, status);
}