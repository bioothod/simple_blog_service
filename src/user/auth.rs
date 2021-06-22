use std::collections::HashMap;

#[derive(Debug)]
struct Meta {
    username: String,
    password: String,
    user_id: usize,
    can_post: bool,
}

impl Meta {
    fn to_user(&self) -> User {
        User {
            username: self.username.to_owned(),
            can_post: self.can_post,
            user_id: self.user_id,
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub user_id: usize,
    pub can_post: bool,
}

impl Clone for User {
    fn clone(&self) -> User {
        User {
            username: self.username.to_owned(),
            user_id: self.user_id,
            can_post: self.can_post,
        }
    }
}


pub struct AuthCtl {
    username2id: HashMap<String, usize>,
    user_ids: HashMap<usize, Meta>,
}

impl AuthCtl {
    pub fn check_password(&self, username: &str, password: &str) -> Result<User, &str> {
        let user_id = match self.username2id.get(username) {
            None => return Err("invalid username/password"),
            Some(user_id) => user_id,
        };

        let meta = match self.user_ids.get(user_id) {
            None => return Err("invalid username/password"),
            Some(meta) => meta,
        };

        match meta.password == password {
            true => Ok(meta.to_user()),
            false => Err("invalid username/password"),
        }
    }

    pub fn check_user_id(&self, user_id: &usize) -> Option<User> {
        match self.user_ids.get(user_id) {
            None => None,
            Some(meta) => Some(meta.to_user()),
        }
    }
}

pub fn init(_auth_path: &str) -> AuthCtl {
    let mut a = AuthCtl {
        username2id: HashMap::new(),
        user_ids: HashMap::new(),
    };

    let x = Meta {
        username: "zbr".to_string(),
        password: "password".to_string(),
        user_id: 2,
        can_post: true,
    };

    a.username2id.insert(x.username.to_owned(), x.user_id);
    a.user_ids.insert(x.user_id, x);

    a
}
