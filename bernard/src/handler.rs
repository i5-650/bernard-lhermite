use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

pub struct Handler<'a> {
    stream: &'a mut TcpStream,
    pub os: String,
    pub addr: String,
}

impl<'a> Handler<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        let mut buffer = [0; 16];
        let mut os = String::new();

        let addr = match stream.peer_addr() {
            Ok(add) => add.to_string(),
            Err(e) => e.kind().to_string(),
        };

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
        // self with lifetime
        Self { stream, os, addr }
    }

    pub fn exec(&mut self, cmd: String) -> Result<(), std::io::Error> {
        match self
            .stream
            .write_all(cmd.as_bytes())
            .map_err(|err| (err.kind(), err))
        {
            Ok(_) => (),
            Err((ErrorKind::BrokenPipe, e)) => {
                println!("Connection closed");
                return Err(e);
            }
            Err((_, e)) => {
                println!("Error writing to stream: {}", e);
                return Ok(());
            }
        }

        let mut buffer = [0; 4096];
        let size = self
            .stream
            .read(&mut buffer)
            .expect("Error reading from stream");

        println!(
            "{}",
            String::from_utf8_lossy(&buffer[..size])
                .trim_end_matches('\0')
                .trim_end()
        );
        Ok(())
    }

    pub fn quit(&mut self) -> Result<(), std::io::Error> {
        self.stream.write_all(b"quit")
    }
}
