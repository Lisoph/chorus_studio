extern crate bincode;
extern crate proto;
extern crate mio;

use std::io::Read;
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
        
        // Catch disconnects
        let cmd = if cmd.is_err() && bytes_read == 0 {
            Ok(proto::Command::Disconnect)
        } else {
            cmd
        };

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
    let mut user_list = HashMap::new();

    let addr = "0.0.0.0:4450".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("TCP listen");
    let poll = Poll::new().expect("Poll");
    poll.register(&listener, LISTENER, Ready::readable(), PollOpt::edge()).expect("Listener register");

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
                    let cmd = {
                        let client = &mut clients.get_mut(&client_id).unwrap();
                        client.read_socket();
                        client.try_deserialize()
                    };
                    if let Some(cmd) = cmd {
                        if let Some(resp) = build_response(cmd, client_id, &mut user_list, &mut clients) {
                            let client = &mut clients.get_mut(&client_id).unwrap();
                            if bincode::serialize_into(&client.stream, &resp).is_err() {
                                println!("Failed to write response!");
                            }
                        }
                    }
                }
            }
        }
    }
}

fn build_response(cmd: proto::Command, client_id: usize, user_list: &mut HashMap<usize, proto::User>, clients: &mut HashMap<usize, ClientSock>) -> Option<proto::Response> {
    use proto::Command::*;
    match cmd {
        ListUsers => Some(proto::Response::UserList(user_list.values().cloned().collect())),
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
                user_list.insert(client_id, proto::User {
                    name: username,
                    status: proto::UserStatus::Avail,
                    in_project: None,
                });

                // Notify other clients about the newly joined guy
                let msg = proto::Response::UserList(user_list.values().cloned().collect());
                for c in clients.values() {
                    let _ = bincode::serialize_into(&c.stream, &msg);
                }

                Some(proto::Response::LoginOk)
            } else {
                Some(proto::Response::LoginInvalid)
            }
        },
        Disconnect => {
            clients.remove(&client_id);
            user_list.remove(&client_id);
            let msg = proto::Response::UserList(user_list.values().cloned().collect());
            for c in clients.values() {
                let _ = bincode::serialize_into(&c.stream, &msg);
            }
            None
        }
    }
}
