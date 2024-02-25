mod clients;
mod utils;

fn main() {
    let os = std::env::consts::FAMILY;
    println!("OS: {}", os);

    let ip = "198.19.249.3";
    let port = "1337";
    match clients::linux::client(ip, port) {
        Ok(_) => println!("Connection established"),
        Err(e) => println!("Error: {}", e),
    }
}
