extern crate bincode;
extern crate proto;

use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::time::Instant;

fn main() {
    let mut clients = Vec::new();
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

    let listener = TcpListener::bind("0.0.0.0:4450").expect("TCP listen");
    listener.set_nonblocking(true).expect("nonblocking socket");

    let mut timer = Instant::now();
    let mut user_count = 0usize;

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
                    let resp = build_response(cmd, user_list.clone());
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
                                true
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
}

fn build_response(cmd: proto::Command, user_list: Vec<proto::User>) -> proto::Response {
    use proto::Command::*;
    match cmd {
        ListUsers => proto::Response::UserList(user_list),
    }
}
