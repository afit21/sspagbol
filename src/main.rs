// Maintained by Afi Hogan https://github.com/afit21
mod services;
use services::{ConfigItem, Service};

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

fn main() {
    print_splash();
    let ci = ConfigItem {
        ciname: "Nginx Host VM".to_string(),
        citype: "Hostmachine".to_string(),
        cidata: vec![
            "192.168.50.79".to_string()
        ]
    };

    let webci = ConfigItem {
        ciname: "Nginx Proxy Manager".to_string(),
        citype: "Webserver".to_string(),
        cidata: vec![
            "https://npm.afih.net/".to_string(),
            "443".to_string()
        ]
    };

    let fakewebci = ConfigItem {
        ciname: "Fake Web Server".to_string(),
        citype: "Webserver".to_string(),
        cidata: vec![
            "https://npmasdada.afih.net/".to_string(),
            "443".to_string()
        ]
    };

    let service = Service {
        name: "Nginx Proxy Manager".to_string(),
        desc: "Proxy server for web servers".to_string(),
        cilist: vec![ci, webci, fakewebci],
    };

    service.print_srv_status();
}
