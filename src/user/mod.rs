use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod config;

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub user_id: usize
}

pub struct UserCtlRaw {
    hash: HashMap<usize, String>,
}
pub type UserCtl = Arc<Mutex<UserCtlRaw>>;

impl UserCtlRaw {
    pub fn new(_cfg: config::Config) -> UserCtlRaw {
        let mut ctl = UserCtlRaw {
            hash: HashMap::new(),
        };

        ctl.push(1, "zbr");
        ctl
    }

    pub fn push(&mut self, user_id: usize, username: &str) {
        self.hash.insert(user_id, username.to_owned());
    }

    pub fn get(&self, user_id: usize) -> Result<User, &str> {
        let username = self.hash.get(&user_id).unwrap();
        Ok(User {
            user_id: user_id,
            username: username.to_string(),
        })
    }
}

pub fn stage(cfg: config::Config) -> UserCtl {
    Arc::new(Mutex::new(UserCtlRaw::new(cfg)))
}
