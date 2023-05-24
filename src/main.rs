use std::net::{self, TcpListener};

mod webhook;

fn main() {
    let addrs = [
        net::SocketAddr::from(([127, 0, 0, 1], 80)),
        net::SocketAddr::from(([127, 0, 0, 1], 443)),
        net::SocketAddr::from(([127, 0, 0, 1], 7878)),
    ];

    let listener = TcpListener::bind(&addrs[..]).unwrap();

    println!(
        "Server Started, Listening on: {:#?}",
        listener.local_addr().unwrap()
    );

    for conn in listener.incoming() {
        let mut conn = conn.unwrap();
        webhook::handle_tcp_connection(&mut conn);
    }
}
