// Maintained by Afi Hogan https://github.com/afit21
//RUSTFLAGS="-Awarnings" cargo run
mod services;
use services::{load_services_from_yaml, print_all_services_in_parallel};

//Splash title
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

    let services = match load_services_from_yaml("config/services.yaml") {
        Ok(srv) => srv,
        Err(e) => {
            eprintln!("Error loading services: {}", e);
            return;
        }
    };

    print_all_services_in_parallel(&services);
}