use crate::user::UserInfo;
use api::{
    basic_types::{ChatIntId, MessageId},
    proto::Message,
};
use bincode::{Decode, Encode};
use std::hash::{Hash, Hasher};

#[derive(Debug, Encode, Decode, Default, Clone)]
pub(crate) struct MessageAddress {
    pub chat_id: ChatIntId,
    pub message_id: MessageId,
}

#[derive(Debug, Encode, Decode, Default)]
pub(crate) struct ChatMessageInfo {
    address: MessageAddress,
    original_address: Option<MessageAddress>,
    pub author_info: Option<UserInfo>,
}

impl ChatMessageInfo {
    pub(crate) fn new(message_id: MessageId) -> Self {
        Self {
            address: MessageAddress {
                chat_id: 0,
                message_id,
            },
            original_address: None,
            author_info: None,
        }
    }

    pub fn address(&self) -> &MessageAddress {
        self.original_address.as_ref().unwrap_or(&self.address)
    }
}

impl Hash for ChatMessageInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.original_address.as_ref() {
            Some(address) => address.message_id.hash(state),
            None => self.address.message_id.hash(state),
        };
    }
}

impl PartialEq for ChatMessageInfo {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.original_address.as_ref(),
            other.original_address.as_ref(),
        ) {
            (Some(lhs), Some(rhs)) => lhs.message_id == rhs.message_id,
            _ => self.address.message_id == other.address.message_id,
        }
    }
}

impl Eq for ChatMessageInfo {}

impl From<&Message> for ChatMessageInfo {
    fn from(message: &Message) -> Self {
        let address = MessageAddress {
            chat_id: message.chat.id,
            message_id: message.message_id,
        };
        let original_address = match (
            message.forward_from_message_id.as_ref(),
            message.forward_from_chat.as_ref(),
        ) {
            (Some(message_id), Some(chat)) => Some(MessageAddress {
                chat_id: chat.id,
                message_id: *message_id,
            }),
            _ => None,
        };
        let user_info = if message.is_forwarded_from_deleted_account() {
            None
        } else {
            message
                .forward_from
                .as_ref()
                .or(message.from.as_ref())
                .map(|u| u.into())
        };
        Self {
            address,
            original_address,
            author_info: user_info,
        }
    }
}
