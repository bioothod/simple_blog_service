use std::sync::{Arc, RwLock};

mod auth;
pub use auth::User;

pub mod config;

pub struct UserCtlRaw {
    pub auth: auth::AuthCtl,
}
pub type UserCtl = Arc<RwLock<UserCtlRaw>>;

impl UserCtlRaw {
    pub fn new(cfg: &config::Config) -> UserCtlRaw {
        let ctl = UserCtlRaw {
            auth: auth::init(cfg.auth_path),
        };
        ctl
    }
}

pub fn init(cfg: &config::Config) -> UserCtl {
    Arc::new(RwLock::new(UserCtlRaw::new(cfg)))
}
