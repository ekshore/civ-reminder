use http;
use httparse;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{io::prelude::*, net};

#[derive(Serialize, Deserialize, Debug)]
struct CivEvent {
    game: String,
    player: String,
    turn: String,
}

impl CivEvent {
    fn from_json(mut event: serde_json::Value) -> CivEvent {
        let game = String::from(event["value1"].take().as_str().unwrap());
        let player = String::from(event["value2"].take().as_str().unwrap());
        let turn = String::from(event["value3"].take().as_str().unwrap());

        CivEvent { game, player, turn }
    }
}

pub fn handle_tcp_connection(conn: &mut net::TcpStream) {
    let request = parse_request(conn);
    let (_parts, body) = request.into_parts();

    if let Media::JSON(body) = body {
        let event = CivEvent::from_json(body);
        dbg!(event);
        conn.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    } else {
        conn.write_all(b"HTTP/1.1 400 BAD REQUEST\r\n\r\n").unwrap();
    }
}

#[derive(Debug)]
enum Media {
    JSON(serde_json::Value),
    Text(String),
    Bytes(Vec<u8>),
    None,
}

fn parse_request(conn: &mut net::TcpStream) -> http::Request<Media> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut request = httparse::Request::new(&mut headers);
    let mut buf = std::io::BufReader::new(conn);
    let mut req_bytes: Vec<u8> = vec![];

    loop {
        let bytes_read = buf.read_until(b'\n', &mut req_bytes).unwrap();
        if bytes_read < 3 {
            break;
        }
    }
    request.parse(req_bytes.as_slice()).unwrap();

    let body: Media;
    if let (Some(content_len), Some(content_type)) =
        get_content_info(request.headers.to_vec().as_ref())
    {
        let mut body_bytes: Vec<u8> = vec![0; content_len];
        buf.read_exact(body_bytes.as_mut_slice()).unwrap();
        body = match content_type {
            b"application/json" => {
                Media::JSON(serde_json::from_slice(body_bytes.as_slice().as_ref()).unwrap())
            }
            b"plain/text" => Media::Text(String::from_utf8(body_bytes).unwrap()),
            b"bytes" => Media::Bytes(body_bytes),
            _ => Media::None,
        }
    } else {
        body = Media::None;
    }

    let mut request_builder = http::Request::builder()
        .uri(request.path.unwrap())
        .method(request.method.unwrap());

    for header in request.headers {
        request_builder = request_builder.header(header.name, header.value);
    }
    request_builder.body(body).unwrap()
}

fn get_content_info<'a>(headers: &'a Vec<httparse::Header>) -> (Option<usize>, Option<&'a [u8]>) {
    let mut content_length: Option<usize> = None;
    let mut content_type: Option<&[u8]> = None;

    for i in 0..headers.len() { 
        if content_type != None && content_length != None {
            break;
        }
        match headers.get(i).expect("Iterating over header index").name {
            "Content-Length" => {
                content_length = Some(convert_ascii_to_num(
                    headers.get(i).expect("Value expected for Key").value,
                ));
            }
            "Content-Type" => {
                content_type = Some(headers.get(i).expect("Value expected for key").value);
            }
            _ => continue,
        }
    }
    (content_length, content_type)
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
