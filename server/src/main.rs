extern crate bincode;
extern crate proto;
extern crate mio;
extern crate rusqlite;

mod db;

use std::io;
use std::io::Read;
use std::time::Duration;
use std::thread;
use std::sync::mpsc;
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

fn ignore_timeout(err: io::Error) -> Option<io::Error> {
    if err.kind() == io::ErrorKind::WouldBlock {
        None
    } else {
        Some(err)
    }
}

fn main() {
    let mut clients: HashMap<usize, ClientSock> = HashMap::new();
    let mut user_list: HashMap<usize, String> = HashMap::new();

    let addr = "0.0.0.0:4450".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("TCP listen");
    let poll = Poll::new().expect("Poll");
    poll.register(&listener, LISTENER, Ready::readable(), PollOpt::edge()).expect("Listener register");

    let mut cur_client_id = 1usize;
    let mut events = Events::with_capacity(1024);

    let database = db::Database::new().expect("Database");
    for u in database.all_users().expect("Query").into_iter() {
        println!("User: {}", u.user_name);
    }

    println!("Enter \"quit\" to quit the server.");

    let (quit_tx, quit_rx) = mpsc::channel();
    let quit_thread = thread::spawn(move || {
        let mut dummy = String::new();
        let _ = io::stdin().read_line(&mut dummy);
        let _ = quit_tx.send(());
    });

    loop {
        if let Ok(()) = quit_rx.try_recv() {
            break;
        }

        if let Err(e) = poll.poll(&mut events, Some(Duration::from_secs(1))) {
            if let Some(e) = ignore_timeout(e) {
                println!("poll ERR: {}", e);
            }
        }

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
                        if let Some(resp) = build_response(&database, cmd, client_id, &mut user_list, &mut clients) {
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

    quit_thread.join().unwrap();
}

fn fetch_users(db: &db::Database, user_list: &HashMap<usize, String>) -> Vec<proto::User> {
    if let Ok(users) = db.users_from_user_name_iter(user_list.values().map(|s| s.as_ref())) {
        users
    } else {
        Vec::new()
    }
}

fn build_response(db: &db::Database, cmd: proto::Command, client_id: usize, user_list: &mut HashMap<usize, String>, clients: &mut HashMap<usize, ClientSock>) -> Option<proto::Response> {
    use proto::Command::*;
    match cmd {
        ListUsers => Some(proto::Response::UserList(fetch_users(db, user_list))),
        Login { email, password, } => {
            let password_hex = {
                use std::fmt::Write;
                let mut buf = String::with_capacity(password.len() * 2);
                for b in password.iter() {
                    let _ = write!(&mut buf, "{:X}", b);
                }
                buf
            };
            println!("Login with email '{}' and password '{}'", email, password_hex);
            if let Ok(Some(user)) = db.user_with_credentials(&email, &password) {
                // We insert the user name instead of the email address, because I want
                // to avoid moving around and possibly leaking user sensitive data.
                user_list.insert(client_id, user.user_name);

                // Notify other clients about the newly joined guy
                let msg = proto::Response::UserList(fetch_users(db, user_list));
                for c in clients.values() {
                    let _ = bincode::serialize_into(&c.stream, &msg);
                }

                Some(proto::Response::LoginOk)
            } else {
                Some(proto::Response::LoginInvalid)
            }
        }
        Disconnect => {
            clients.remove(&client_id);
            user_list.remove(&client_id);
            let msg = proto::Response::UserList(fetch_users(db, user_list));
            for c in clients.values() {
                let _ = bincode::serialize_into(&c.stream, &msg);
            }
            None
        }
    }
}
