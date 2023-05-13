use httparse;
use std::{
    io::prelude::*,
    net::{self, TcpListener},
};

fn main() {
    let addr = net::Ipv4Addr::new(127, 0, 0, 1);
    let port = 7878;
    let socket_addr = net::SocketAddrV4::new(addr, port);

    let listener = TcpListener::bind(socket_addr).unwrap();

    for conn in listener.incoming() {
        let mut conn = conn.unwrap();
        handle_request(&mut conn);
    }
}

fn handle_request(req: &mut net::TcpStream) {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut request = httparse::Request::new(&mut headers);
    let mut buf_reader = std::io::BufReader::new(req);

    let mut buff: Vec<u8> = vec![];

    loop {
        let bytes_read = buf_reader.read_until(b'\n', &mut buff).unwrap();
        if bytes_read < 3 {
            break;
        }
    }
    request.parse(buff.as_slice()).unwrap();

    let content_length = request.headers.last().unwrap().value;
    let content_length = convert_ascii_to_num(content_length);

    println!("Request: {:?}", &request);
    
    let mut body: Vec<u8> = vec![0; content_length];
     
    buf_reader.read_exact(body.as_mut_slice()).unwrap();
    let body_text = String::from_utf8(body).unwrap();
    println!("Request Body: {:#?}", body_text);
    ()
}

fn convert_ascii_to_num(val: &[u8]) -> usize {
    let mut num: usize = 0;
    for i in 0..val.len() {
        let ten: usize = 10;
        let place: usize = val.len() - (i + 1);
        let place = ten.pow(place as u32);
        num += place * (val[i] - 48) as usize;
    }
   num
}
