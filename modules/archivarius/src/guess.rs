use api::basic_types::{MessageId, UserId};
use bincode::{Decode, Encode};
use std::collections::HashMap;

#[derive(Encode, Decode, Debug, Default)]
pub(crate) struct ChatGuessInfo {
    pub points: HashMap<UserId, usize>,
    pub message_id: Option<MessageId>,
}

impl ChatGuessInfo {
    fn add_point(&mut self, id: UserId) {
        self.points
            .entry(id)
            .and_modify(|score| *score += 1)
            .or_insert(1);
    }

    pub fn finish_game(&mut self, winner_id: UserId) {
        self.add_point(winner_id);
        self.message_id = None;
    }
}
