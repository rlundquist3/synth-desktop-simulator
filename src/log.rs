use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_LOG_LINES: usize = 200;

static LOGS: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn logs() -> &'static Mutex<VecDeque<String>> {
    LOGS.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn push(message: impl Into<String>) {
    let mut logs = logs().lock().unwrap();
    logs.push_back(message.into());
    if logs.len() > MAX_LOG_LINES {
        logs.pop_front();
    }
}

pub fn snapshot() -> Vec<String> {
    logs().lock().unwrap().iter().cloned().collect()
}
