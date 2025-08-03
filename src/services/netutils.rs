use std::process::Command;
use reqwest::blocking::Client;
use std::time::Duration;
use url::Url;

//Attempts to ping a service
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