extern crate chrono;
use std::{
    net::{self, TcpListener},
    sync::{Arc, Mutex},
};
extern crate timer;

mod webhook;

fn main() {
    let addrs = [
        net::SocketAddr::from(([127, 0, 0, 1], 80)),
        net::SocketAddr::from(([127, 0, 0, 1], 443)),
        net::SocketAddr::from(([127, 0, 0, 1], 7878)),
    ];

    let listener = TcpListener::bind(&addrs[..]).unwrap();
    let webhook = webhook::WebHook::new();

    println!(
        "Server Started, Listening on: {:#?}",
        listener.local_addr().unwrap()
    );

    let webhook = Arc::new(Mutex::new(webhook));

    let timer = timer::Timer::new();
    let r_webhook = Arc::clone(&webhook);
    let _schedule = timer.schedule_repeating(chrono::Duration::hours(12), move || {
        r_webhook.lock().unwrap().send_reminder();
    });

    for conn in listener.incoming() {
        if let Ok(mut conn) = conn {
            let _ = &webhook.lock().unwrap().handle_tcp_connection(&mut conn);
        }
    }
}
