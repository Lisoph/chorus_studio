extern crate serde;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
pub enum Command {
    ListUsers,
    Login { username: String, password: Vec<u8> },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    UserList(Vec<User>),
    LoginOk,
    LoginInvalid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub status: UserStatus,
    pub in_project: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum UserStatus {
    Avail,
    Away,
    Offline,
}
