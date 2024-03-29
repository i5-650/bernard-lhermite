use std::process::exit;

use fork::{fork, Fork};

mod clients;
mod utils;

fn main() {
    /*match fork() {
    Ok(Fork::Child) => match clients::linux::client("198.19.249.3", "1337") {
        Ok(_) => println!("Connection established"),
        Err(e) => println!("Error: {}", e),
    },
    Ok(Fork::Parent(_)) => exit(0),
    Err(e) => {
        println!("Error: {}", e);
        exit(1);
    }
    }*/
    match clients::linux::client("198.19.249.3", "1337") {
        Ok(_) => println!("Connection established"),
        Err(e) => println!("Error: {}", e),
    }
}
