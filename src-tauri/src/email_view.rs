use std::sync::Mutex;

pub struct EmailViewState {
    pub html: Mutex<Option<String>>,
    pub allow_external: Mutex<bool>,
}

impl Default for EmailViewState {
    fn default() -> Self {
        Self {
            html: Mutex::new(None),
            allow_external: Mutex::new(false),
        }
    }
}
