use std::sync::Mutex;
use crate::recording::Recording;

pub struct AppState {
    pub recorder: Mutex<Recording>,
}
