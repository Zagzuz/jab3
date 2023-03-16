use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub type UpdateId = i64;
pub type UserId = i64;
pub type ChatIntId = i64;
pub type MessageId = i32;
pub type MessageThreadId = i32;

#[derive(Debug, Deserialize, Serialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        Self::from_secs(secs)
    }

    pub fn from_secs(secs: u64) -> Self {
        Timestamp(secs)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp::now()
    }
}
