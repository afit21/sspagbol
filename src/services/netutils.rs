use std::process::Command;
use reqwest::blocking::Client;
use std::time::{SystemTime, Duration};
use url::Url;
use native_tls::TlsConnector;
use std::net::{TcpStream};
use std::io::{Read};
use std::net::UdpSocket;
use std::net::ToSocketAddrs;
use x509_parser::prelude::*;

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

//Checks for SSL Certificate Validity. Returns a String with the result
pub fn check_ssl_cert(base_url: &str, port: u16) -> String {
    let host = match url::Url::parse(base_url) {
        Ok(url) => match url.host_str() {
            Some(h) => h.to_string(),
            None => return "❌ - Invalid URL".to_string(),
        },
        Err(_) => return "❌ - Invalid URL".to_string(),
    };

    let connector = match TlsConnector::new() {
        Ok(c) => c,
        Err(_) => return "❌ - Failed to create TLS connector".to_string(),
    };

    let addr = format!("{}:{}", host, port);
    let addrs = match addr.to_socket_addrs() {
        Ok(a) => a,
        Err(_) => return "❌ - Failed to resolve address".to_string(),
    };

    for addr in addrs {
        if let Ok(stream) = std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
            if let Ok(tls_stream) = connector.connect(&host, stream) {
                if let Ok(Some(cert)) = tls_stream.peer_certificate() {
                    if let Ok(cert_der) = cert.to_der() {
                        if let Ok((_, parsed_cert)) = parse_x509_certificate(&cert_der) {
                            let not_after = parsed_cert.tbs_certificate.validity.not_after;
                            let expiry = not_after.to_datetime();


                            let expiry_system_time: SystemTime = expiry.into();
                            let now = SystemTime::now();

                            if expiry_system_time < now {
                                return "❌ - Expired".to_string();
                            } else {
                                let remaining = expiry_system_time.duration_since(now)
                                    .unwrap_or(Duration::ZERO);
                                let days = remaining.as_secs() / 86400;
                                return format!("✅ - Valid For {} days", days);
                            }
                        }
                    }
                }
                return "❌ - SSL Certificate Missing or Unreadable".to_string();
            }
        }
    }

    "❌ - Unable to connect to server".to_string()
}

//Checks if SSH Server is up. Returns a String with the result
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

// Checks if a DNS server is up by sending a minimal DNS query
pub fn dns_server_up(host: &str, domain: &str) -> String {
    let server = format!("{}:53", host); // DNS servers listen on port 53 UDP

    // Create UDP socket bound to any local port
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return "❌".to_string(),
    };

    // Set timeout so we don't hang
    if socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .is_err()
    {
        return "❌".to_string();
    }

    // Encode the domain into DNS label format
    let mut qname = Vec::new();
    for part in domain.split('.') {
        if part.len() > 0x3F {
            return "❌".to_string(); // label too long
        }
        qname.push(part.len() as u8);
        qname.extend_from_slice(part.as_bytes());
    }
    qname.push(0); // null terminator for domain

    // Build a minimal DNS query
    let mut query = vec![
        0x12, 0x34, // Transaction ID
        0x01, 0x00, // Flags: standard query
        0x00, 0x01, // Questions: 1
        0x00, 0x00, // Answer RRs
        0x00, 0x00, // Authority RRs
        0x00, 0x00, // Additional RRs
    ];
    query.extend_from_slice(&qname);
    query.extend_from_slice(&[0x00, 0x01]); // Type: A
    query.extend_from_slice(&[0x00, 0x01]); // Class: IN

    // Send query to DNS server
    if socket.send_to(&query, &server).is_err() {
        return "❌".to_string();
    }

    let mut buf = [0u8; 512];
    match socket.recv_from(&mut buf) {
        Ok((size, _)) if size > 0 => {
            // Check if Transaction ID matches
            if buf[0] == 0x12 && buf[1] == 0x34 {
                return "✅".to_string();
            }
            "❌".to_string()
        }
        _ => "❌".to_string(),
    }
}
