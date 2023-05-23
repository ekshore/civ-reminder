use http;
use httparse;
use std::{io::prelude::*, net};

pub fn handle_tcp_connection(conn: &mut net::TcpStream) {
    let request = parse_request(conn);
    let (parts, body) = request.into_parts();

    let mut body_text: Option<String> = None;
    if let Some(content_type) = parts.headers.get("Content-Type") {
        if content_type.eq("application/json") {
            body_text = Some(
                String::from_utf8(
                    body.expect("Media type is Application/JSON what body was expected"),
                )
                .unwrap(),
            );
        }
    }
    dbg!(body_text);
}

fn parse_request(conn: &mut net::TcpStream) -> http::Request<Option<Vec<u8>>> {
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

    let mut body: Option<Vec<u8>> = None;
    if let Some(content_len) = get_content_length(request.headers.to_vec().as_ref()) {
        let mut body_bytes: Vec<u8> = vec![0; content_len];
        buf.read_exact(body_bytes.as_mut_slice()).unwrap();
        body = Some(body_bytes);
    }

    let mut request_builder = http::Request::builder()
        .uri(request.path.unwrap())
        .method(request.method.unwrap());

    for header in request.headers {
        request_builder = request_builder.header(header.name, header.value);
    }

    request_builder.body(body).unwrap()
}

fn get_content_length(headers: &Vec<httparse::Header>) -> Option<usize> {
    let mut content_length_header: Option<httparse::Header> = None;
    for i in 0..headers.len() {
        if "Content-Length".eq_ignore_ascii_case(headers.get(i).unwrap().name) {
            content_length_header = Some(
                *headers
                    .get(i)
                    .expect("This is a checked index, something is very wrong"),
            );
            break;
        }
    }
    if let Some(content_length_header) = content_length_header {
        let content_length = convert_ascii_to_num(content_length_header.value);
        return Some(content_length);
    }
    None
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
