extern crate bincode;
extern crate proto;
extern crate mio;

use std::io::{Read, ErrorKind};
use std::time::Instant;
use std::collections::HashMap;

use mio::net::{TcpListener, TcpStream};
use mio::{Poll, Token, Ready, PollOpt, Events};

const LISTENER: Token = Token(0);

struct ClientSock {
    stream: TcpStream,
    data_buf: Vec<u8>,
}

impl ClientSock {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            data_buf: Vec::with_capacity(1024),
        }
    }

    fn read_socket(&mut self) {
        let _ = self.stream.read_to_end(&mut self.data_buf);
    }

    fn try_deserialize(&mut self) -> Option<proto::Command> {
        let (cmd, bytes_read) = {
            let mut bytes = self.data_buf.as_slice();
            let cmd = bincode::deserialize_from(&mut bytes);
            let bytes_read = bytes.as_ptr() as usize - self.data_buf.as_slice().as_ptr() as usize;
            (cmd, bytes_read)
        };
        println!("try_deserialize: {} bytes", bytes_read);
        match cmd {
            Ok(cmd) => {
                self.data_buf.drain(0..bytes_read);
                Some(cmd)
            }
            Err(e) => {
                match *e {
                    bincode::ErrorKind::Io(..) => {} // Not enough data received yet
                    _ => { self.data_buf.drain(0..bytes_read); },
                };
                None
            }
        }
    }
}

fn main() {
    let mut clients = HashMap::new();
    let mut user_list = vec![
        proto::User {
            name: "Lisoph".to_owned(),
            status: proto::UserStatus::Avail,
            in_project: Some("Archon".to_owned()),
        },
        proto::User {
            name: "Irockus".to_owned(),
            status: proto::UserStatus::Away,
            in_project: None,
        },
    ];

    let addr = "0.0.0.0:4450".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("TCP listen");
    let poll = Poll::new().expect("Poll");
    poll.register(&listener, LISTENER, Ready::readable(), PollOpt::edge()).expect("Listener register");

    let mut timer = Instant::now();
    let mut user_count = 0usize;
    let mut cur_client_id = 1usize;

    let mut events = Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for e in events.iter() {
            match e.token() {
                LISTENER => {
                    let (client_stream, client_addr) = listener.accept().expect("Client accept");
                    println!("New client: {:?}", client_addr);
                    let client = ClientSock::new(client_stream);
                    clients.insert(cur_client_id, client);
                    poll.register(&clients[&cur_client_id].stream, Token(cur_client_id), Ready::readable(), PollOpt::edge()).expect("Client register");
                    cur_client_id += 1;
                },
                Token(client_id) => {
                    let mut client = &mut clients.get_mut(&client_id).unwrap();
                    client.read_socket();
                    if let Some(cmd) = client.try_deserialize() {
                        let resp = build_response(cmd, &mut user_list);
                        if bincode::serialize_into(&mut client.stream, &resp).is_err() {
                            println!("Failed to write response!");
                        }
                    }
                }
            }
        }
    }

    /*
    loop {
        if let Ok((client, _)) = listener.accept() {
            let client_addr = client
                .peer_addr()
                .or_else(|_| client.local_addr())
                .expect("New client IP");
            println!("New connection: {}", client_addr);
            clients.push(client);
        }

        if timer.elapsed().as_secs() >= 15 {
            user_list.push(proto::User {
                name: format!("Extra User #{:02}", user_count + 1),
                status: proto::UserStatus::Avail,
                in_project: None,
            });
            user_count += 1;
            timer = Instant::now();

            for c in clients.iter_mut() {
                if bincode::serialize_into(c, &proto::Response::UserList(user_list.clone()))
                    .is_err()
                {
                    println!("Failed to write response!");
                }
            }
        }

        clients.retain(|client| {
            let client: &mut TcpStream = unsafe { &mut *(client as *const _ as *mut _) };
            let mut buf = [0u8];
            let is_disconnected = client.peek(&mut buf).ok() == Some(0);
            if is_disconnected {
                println!("Client disconnected");
                return false;
            }

            let cmd: bincode::Result<proto::Command> =
                bincode::deserialize_from(client as &mut std::io::Read);
            match cmd {
                Ok(cmd) => {
                    let resp = build_response(cmd, &mut user_list);
                    if bincode::serialize_into(client, &resp).is_err() {
                        println!("Failed to write response!");
                    }
                    true
                }
                Err(b) => {
                    if let bincode::ErrorKind::Io(e) = *b {
                        match e.kind() {
                            ErrorKind::WouldBlock => true,
                            ErrorKind::UnexpectedEof => true,
                            ErrorKind::ConnectionAborted => {
                                println!("Client disconnected");
                                false
                            }
                            _ => {
                                println!("Unhandled error: {:?}", e);
                                false
                            }
                        }
                    } else {
                        println!("{:?}", *b);
                        true
                    }
                }
            }
        });

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    */
}

fn build_response(cmd: proto::Command, user_list: &mut Vec<proto::User>) -> proto::Response {
    use proto::Command::*;
    match cmd {
        ListUsers => proto::Response::UserList(user_list.clone()),
        Login { username, password, } => {
            let password_hex = {
                use std::fmt::Write;
                let mut buf = String::with_capacity(password.len() * 2);
                for b in password.iter() {
                    let _ = write!(&mut buf, "{:X}", b);
                }
                buf
            };
            println!("Login with username '{}' and password '{}'", username, password_hex);
            if password == [0xc0u8, 0x06, 0x7d, 0x4a, 0xf4, 0xe8, 0x7f, 0x00, 0xdb, 0xac, 0x63, 0xb6, 0x15, 0x68, 0x28, 0x23, 0x70, 0x59, 0x17, 0x2d, 0x1b, 0xbe, 0xac, 0x67, 0x42, 0x73, 0x45, 0xd6, 0xa9, 0xfd, 0xa4, 0x84] {
                user_list.push(proto::User {
                    name: username,
                    status: proto::UserStatus::Avail,
                    in_project: None,
                });
                proto::Response::LoginOk
            } else {
                proto::Response::LoginInvalid
            }
        },
    }
}
