#[macro_export]
macro_rules! listener {
    ($ip:expr, $port:expr) => {{
        use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
        let ip = $ip.parse::<Ipv4Addr>().expect("Invalid IP address");
        let port = $port.parse::<u16>().expect("Invalid port");
        let addr = SocketAddrV4::new(ip, port);
        let listener = TcpListener::bind(addr).expect("Failed to bind to address");
        listener
    }};
}

#[macro_export]
macro_rules! prompt {
    ($handler:expr) => {{
        use std::io::Write;
        let prompt = format!("{}-{}-CMD# ", $handler.os, $handler.addr);
        print!("{}", prompt.blue());
        std::io::stdout().flush().expect("Failed to flush");
        let mut cmd = String::new();
        std::io::stdin().read_line(&mut cmd).unwrap();
        cmd
    }};
}
