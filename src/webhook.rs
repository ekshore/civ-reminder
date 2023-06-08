use discord;
use std::env;
use http;
use httparse;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, io::prelude::*, net};

struct Player {
    pub game_name: String,
    pub discord_user: discord::model::User,
}

#[allow(dead_code)]
pub struct WebHook {
    discord_client: discord::Discord,
    discord_server: discord::model::ServerInfo,
    discord_channel: discord::model::PublicChannel,
    pending_player: Option<usize>,
    players: Vec<Player>,
}

impl WebHook {
    pub fn new() -> WebHook {
        let bot_token = env::var("CIV_REMINDER_TOKEN").expect("Token needs to be stored under `CIV_REMINDER_TOKEN`");
        let discord_client = discord::Discord::from_bot_token(bot_token.as_str())
            .expect("Is my token Incorrect?");
        let servers = discord_client
            .get_servers()
            .expect("This Bot should be added to a server");
        let discord_server = servers.get(0).take().unwrap().to_owned();

        println!(
            "Discord Client created Successfully, connected to the {} server",
            discord_server.name
        );

        let discord_channels = discord_client
            .get_server_channels(discord_server.id)
            .expect("A server should have channels");
        let civ_reminder_channel = discord_channels
            .iter()
            .filter(|channel| channel.id.to_string().eq("1093982612470636574"))
            .next()
            .unwrap()
            .to_owned();

        println!(
            "Found {} channel for civ notifications",
            civ_reminder_channel.name
        );

        let members = discord_client
            .get_server_members(discord_server.id)
            .expect("A server should have members");

        let players = build_player_list(members.iter().map(|m| m.user.to_owned()).collect());

        println!("Built Player list:");
        for i in 0..players.len() {
            println!(
                " - Name: {}, Discord Id: {}",
                players[i].game_name, players[i].discord_user.id
            );
        }

        WebHook {
            discord_client,
            discord_server,
            discord_channel: civ_reminder_channel,
            players,
            pending_player: None,
        }
    }

    pub fn handle_tcp_connection(&mut self, conn: &mut net::TcpStream) {
        let request = parse_request(conn);
        dbg!(&request);
        let (_parts, body) = request.into_parts();

        if let Media::JSON(body) = body {
            let event = CivEvent::from_json(body);
            self.handle_event(event);
            conn.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        } else {
            conn.write_all(b"HTTP/1.1 400 BAD REQUEST\r\n\r\n").unwrap();
        }
    }

    fn handle_event(&mut self, event: CivEvent) {
        dbg!(&event);
        if let Some(idx) = self.pending_player {
            if self.players[idx]
                .game_name
                .eq_ignore_ascii_case(event.player.as_str())
            {
                return ();
            }
        }

        self.pending_player = {
            let mut player: Option<usize> = None;
            for i in 0..self.players.len() {
                if event
                    .player
                    .as_str()
                    .eq_ignore_ascii_case(self.players[i].game_name.as_str())
                {
                    player = Some(i);
                    break;
                }
            }
            player
        };
        self.send_reminder();
    }

    pub fn send_reminder(&self) {
        if let Some(idx) = self.pending_player {
            println!("Sending turn reminder to {}", self.players[idx].game_name);
            let reminder_message = format!(
                "{} it is your turn in the civ game. People are waiting on you!",
                self.players[idx].discord_user.mention()
            );
            self.discord_client
                .send_message(
                    self.discord_channel.id,
                    reminder_message.as_str(),
                    "",
                    false,
                )
                .unwrap();
        }
        println!("Reminder processed");
    }
}

fn build_player_list(users: Vec<discord::model::User>) -> Vec<Player> {
    let game_players: HashMap<&str, String> = HashMap::from([
        ("Ekshore", String::from("Ekshore")),
        ("BlazeGemSpark", String::from("BlazeGemSpark")),
        ("J_Storm", String::from("J_Strohm")),
        ("Galloran92", String::from("GalloranTBK")),
        ("Heavy\"Spike\"-782", String::from("Heavy119"))
    ]);
    users
        .iter()
        .filter(|usr| game_players.contains_key(&usr.name.as_str()))
        .map(|usr| Player {
            game_name: game_players.get(usr.name.as_str()).unwrap().to_owned(),
            discord_user: usr.to_owned(),
        })
        .collect()
}

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
