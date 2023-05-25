use std::net::{self, TcpListener};

mod webhook;

fn main() {
    let addrs = [
        net::SocketAddr::from(([127, 0, 0, 1], 80)),
        net::SocketAddr::from(([127, 0, 0, 1], 443)),
        net::SocketAddr::from(([127, 0, 0, 1], 7878)),
    ];

    let listener = TcpListener::bind(&addrs[..]).unwrap();
    let mut webhook = webhook::WebHook::new();

    println!(
        "Server Started, Listening on: {:#?}",
        listener.local_addr().unwrap()
    );

    for conn in listener.incoming() {
        if let Ok(mut conn) = conn {
            webhook.handle_tcp_connection(&mut conn);
        }
    }
}
