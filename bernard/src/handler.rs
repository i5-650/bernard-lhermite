use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;

pub struct Handler<'a> {
    stream: &'a mut TcpStream,
    pub os: String,
    pub addr: String,
}

impl<'a> Handler<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        let addr = match stream.peer_addr() {
            Ok(add) => add.to_string(),
            Err(e) => e.kind().to_string(),
        };

        let mut buffer = [0; 16];
        let mut os = "Unknown".to_string();
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

        let mut reader = BufReader::new(&mut self.stream);
        let buffer = reader.fill_buf()?.to_vec();
        reader.consume(buffer.len());

        println!(
            "{}",
            String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .trim_end()
        );
        Ok(())
    }

    pub fn quit(&mut self) -> Result<(), std::io::Error> {
        self.stream.write_all(b"quit")
    }

    pub fn download(&mut self, cmd: String, filename: String) -> Result<(), std::io::Error> {
        self.stream.write_all(cmd.as_bytes())?;

        let mut file = self.create_file_if_not_exists(filename.as_str())?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut buff = reader.fill_buf()?.to_vec();
        reader.consume(buff.len());

        file.write_all(&buff)?;
        match buff.flush() {
            Ok(_) => (),
            Err(e) => {
                println!("Error flushing file: {}", e);
            }
        }

        println!("File received");
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
