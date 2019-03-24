extern crate serde;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
pub enum Command {
    ListUsers,
    Login { email: String, password: Vec<u8> },
    Disconnect,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    UserList(Vec<User>),
    LoginOk,
    LoginInvalid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub user_name: String,
    pub activity: UserActivity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserActivity {
    Offline,
    Away,
    Active,
    InProject(String),
}
