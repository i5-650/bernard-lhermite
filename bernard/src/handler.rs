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

    pub fn download(&mut self, cmd: String, filename: String) -> Result<(), std::io::Error> {
        let status = self.stream.write_all(cmd.as_bytes());
        if let Err(e) = status {
            return Err(e);
        }

        let mut file = match self.create_file_if_not_exists(filename.as_str()) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        let mut buffer = [0; 4096];
        let mut end = false;
        while !end {
            let size = self
                .stream
                .read(&mut buffer)
                .expect("Error reading from stream");
            if size < 4096 {
                end = true;
            }
            match file.write_all(&buffer[..size]) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        file.flush()
    }

    fn create_file_if_not_exists(&self, filename: &str) -> Result<std::fs::File, std::io::Error> {
        if std::path::Path::new(filename).exists() {
            Err(std::io::Error::new(
                ErrorKind::AlreadyExists,
                "File already exists",
            ))
        } else {
            match std::fs::File::create(filename) {
                Ok(f) => Ok(f),
                Err(e) => Err(e),
            }
        }
    }
}
