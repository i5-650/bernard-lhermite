use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub enum HandlerStatus {
    Ok,
    FileError,
    CmdError,
    EndSession,
    SendError,
}

pub fn dl_cmd(cmd: &String, tls_stream: &mut TcpStream) -> HandlerStatus {
    let path: Vec<&str> = cmd.split(' ').collect();
    match File::open(path[1]) {
        Ok(mut file) => match read_file(&mut file) {
            Ok(vec) => {
                let status = tls_stream.write_all(&vec);
                if status.is_err() {
                    return HandlerStatus::SendError;
                }
                HandlerStatus::Ok
            }
            Err(_) => HandlerStatus::FileError,
        },
        Err(_) => HandlerStatus::FileError,
    }
}

pub fn read_file(file: &mut File) -> Result<Vec<u8>, std::io::Error> {
    let mut reader = BufReader::new(file);
    let vec = reader.fill_buf()?.to_vec();
    reader.consume(vec.len());
    Ok(vec)
}
