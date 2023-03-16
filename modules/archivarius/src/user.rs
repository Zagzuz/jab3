use api::{basic_types::UserId, proto::User};
use bincode::{Decode, Encode};
use std::hash::{Hash, Hasher};

#[derive(Encode, Decode, Debug, Default, Clone)]
pub(crate) struct UserInfo {
    pub id: UserId,
    pub full_name: String,
    pub username: Option<String>,
}

impl Hash for UserInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for UserInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for UserInfo {}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            full_name: user.full_name().into(),
            username: user.username.clone().map(|s| s.to_string()),
        }
    }
}

impl PartialEq<str> for UserInfo {
    fn eq(&self, answer: &str) -> bool {
        if let Ok(id) = answer.parse::<UserId>() {
            if id == self.id {
                return true;
            }
        }
        if self.full_name.to_lowercase() == answer.to_lowercase() {
            return true;
        }
        if let Some(username) = self.username.as_ref() {
            if username.to_lowercase() == answer.to_lowercase() {
                return true;
            }
        }
        false
    }
}
