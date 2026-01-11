use lazy_static::lazy_static;
use std::sync::{Mutex, Once};

lazy_static! {
    pub static ref OAUTH_PORT: Mutex<u16> = Mutex::new(0);
}

static INIT: Once = Once::new();

pub fn set_oauth_port(port: u16) {
    INIT.call_once(|| {
        *OAUTH_PORT.lock().unwrap() = port;
    });
}

pub fn get_oauth_port() -> u16 {
    *OAUTH_PORT.lock().unwrap()
}

