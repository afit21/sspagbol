use std::process::Command;
use reqwest::blocking::Client;
use std::time::Duration;
use url::Url;
use std::net::{TcpStream};
use std::io::{Read};

//Attempts to ping a service. Returns true or false
pub fn ping(target: &str) -> bool {
    //Windows Implementation
    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(&["-n", "1", target])
            .output()
    } else {
        //Linux Implementation
        Command::new("ping")
            .args(&["-c", "1", target])
            .output()
    };

    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

//Checks for HTTP response. Returns true or false
pub fn web_server_up(base_url: &str, port: u16) -> bool {
    // Parse and override the port in the URL
    let mut url = match Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    url.set_port(Some(port)).ok(); // ignores errors (e.g., invalid port)

    let client = match Client::builder()
        .timeout(Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    match client.get(url).send() {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

pub fn ssh_server_up(host: &str, port: u16) -> String {
    let address = format!("{host}:{port}");
    // Attempt to connect with a timeout
    let Ok(mut stream) = TcpStream::connect_timeout(
        &address.parse().unwrap(),
        Duration::from_secs(3),
    ) else {
        return "❌".to_string();
    };

    // Set a read timeout for the banner
    if stream.set_read_timeout(Some(Duration::from_secs(2))).is_err() {
        return "❌".to_string();
    }

    // Read the server's banner (first message)
    let mut buffer = [0u8; 256];
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            let banner = String::from_utf8_lossy(&buffer[..size]);
            
            if banner.starts_with("SSH-") {
                return format!("✅ - {}", banner.to_string().trim_end_matches(&['\r', '\n'][..]));
            } else {
                return "❌".to_string();
            }
        }
        _ => "❌".to_string(),
    }
}
