use compact_str::CompactString;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_json::Map;
use serde_with::skip_serializing_none;

use crate::basic_types::{ChatIntId, MessageId, Timestamp, UpdateId, UserId};

// fixme: Date the change was done in Unix time
pub type Date = u64;

/// This object represents the contents of a file to be uploaded. Must be posted using multipart/form-data in the usual way that files are uploaded via the browser.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum InputFile {
    /// FileID is an ID of a file already uploaded to Telegram.
    FileID(CompactString),
    /// FileURL is a URL to use as a file for a request.
    FileURL(CompactString),
    /// fileAttach is an internal file type used for processed media groups.
    FileAttach(CompactString),
    /// FileBytes contains information about a set of bytes to upload as a File.
    FileBytes(CompactString, Vec<u8>),
    /// FilePath is a path to a local file.
    FilePath(CompactString),
}
/// On success,returns a InputFileResult object data method

pub enum InputFileResult {
    /// don't need upload
    Text(CompactString),
    /// must upload using multipart/form-data
    Part(reqwest::multipart::Part),
}

impl InputFile {
    pub fn need_upload(&self) -> bool {
        matches!(self, InputFile::FileBytes(_, _) | InputFile::FilePath(_))
    }

    pub async fn data(&self) -> eyre::Result<InputFileResult> {
        match self {
            InputFile::FileID(id) => Ok(InputFileResult::Text(id.clone())),
            InputFile::FileURL(url) => Ok(InputFileResult::Text(url.clone())),
            InputFile::FileAttach(attach) => Ok(InputFileResult::Text(attach.clone())),
            InputFile::FileBytes(file_name, bytes) => Ok(InputFileResult::Part(
                reqwest::multipart::Part::bytes(bytes.clone()).file_name(file_name.to_string()),
            )),
            InputFile::FilePath(path) => Ok(InputFileResult::Part(
                reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(
                    tokio_util::codec::FramedRead::new(
                        tokio::fs::File::open(path.as_str()).await?,
                        tokio_util::codec::BytesCodec::new(),
                    ),
                ))
                .file_name(path.to_string()),
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UpdateType {
    Message,
    EditedMessage,
    ChannelPost,
    EditedChannelPost,
    InlineQuery,
    ChosenInlineResult,
    CallbackQuery,
    ShippingQuery,
    PreCheckoutQuery,
    Poll,
    PollAnswer,
    MyChatMember,
    ChatMember,
    ChatJoinRequest,
}

#[derive(Debug)]
pub struct CommonUpdate {
    pub id: UpdateId,
    pub data: Update,
}

#[derive(Debug)]
pub enum Update {
    MessageUpdate(Message),
    EditedMessageUpdate(Message),
    ChannelPostUpdate(Message),
    EditedChannelPostUpdate(Message),
    InlineQueryUpdate(InlineQuery),
    ChosenInlineResultUpdate(ChosenInlineResult),
    CallbackQueryUpdate(CallbackQuery),
    ShippingQueryUpdate(ShippingQuery),
    PreCheckoutQueryUpdate(PreCheckoutQuery),
    PollUpdate(Poll),
    PollAnswerUpdate(PollAnswer),
    MyChatMemberUpdate(ChatMemberUpdated),
    ChatMemberUpdate(ChatMemberUpdated),
    ChatJoinRequestUpdate(ChatJoinRequest),
}

impl<'de> Deserialize<'de> for CommonUpdate {
    fn deserialize<D>(deserializer: D) -> Result<CommonUpdate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = Map::deserialize(deserializer)?;

        let id = map
            .remove("update_id")
            .ok_or_else(|| de::Error::missing_field("update_id"))
            .map(Deserialize::deserialize)?
            .map_err(de::Error::custom)?;

        let (key, value) = map
            .into_iter()
            .next()
            .ok_or_else(|| de::Error::custom("update with no data"))?;

        let update =
            match key.as_str() {
                "message" => serde_json::from_value::<Message>(value).map(Update::MessageUpdate),
                "edited_message" => {
                    serde_json::from_value::<Message>(value).map(Update::EditedMessageUpdate)
                }
                "channel_post" => {
                    serde_json::from_value::<Message>(value).map(Update::ChannelPostUpdate)
                }
                "edited_channel_post" => {
                    serde_json::from_value::<Message>(value).map(Update::EditedChannelPostUpdate)
                }
                "inline_query" => {
                    serde_json::from_value::<InlineQuery>(value).map(Update::InlineQueryUpdate)
                }
                "chosen_inline_result" => serde_json::from_value::<ChosenInlineResult>(value)
                    .map(Update::ChosenInlineResultUpdate),
                "callback_query" => {
                    serde_json::from_value::<CallbackQuery>(value).map(Update::CallbackQueryUpdate)
                }
                "shipping_query" => {
                    serde_json::from_value::<ShippingQuery>(value).map(Update::ShippingQueryUpdate)
                }
                "pre_checkout_query" => serde_json::from_value::<PreCheckoutQuery>(value)
                    .map(Update::PreCheckoutQueryUpdate),
                "poll" => serde_json::from_value::<Poll>(value).map(Update::PollUpdate),
                "poll_answer" => {
                    serde_json::from_value::<PollAnswer>(value).map(Update::PollAnswerUpdate)
                }
                "my_chat_member" => serde_json::from_value::<ChatMemberUpdated>(value)
                    .map(Update::MyChatMemberUpdate),
                "chat_member" => {
                    serde_json::from_value::<ChatMemberUpdated>(value).map(Update::ChatMemberUpdate)
                }
                "chat_join_request" => serde_json::from_value::<ChatJoinRequest>(value)
                    .map(Update::ChatJoinRequestUpdate),
                _ => {
                    return Err(de::Error::custom("unknown update"));
                }
            }
            .map_err(de::Error::custom)?;
        Ok(CommonUpdate { id, data: update })
    }
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct InlineQuery {
    pub id: CompactString,
    pub from: User,
    pub query: CompactString,
    pub offset: CompactString,
    pub chat_type: Option<ChatType>,
    pub location: Option<Location>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ChosenInlineResult {
    pub result_id: i64,
    pub from: User,
    pub location: Option<Location>,
    pub inline_message_id: Option<CompactString>,
    pub query: Option<CompactString>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub id: CompactString,
    pub from: User,
    pub message: Option<Message>,
    pub inline_message_id: Option<CompactString>,
    pub chat_instance: Option<CompactString>,
    pub data: Option<CompactString>,
    pub game_short_name: Option<CompactString>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ShippingQuery {
    pub id: CompactString,
    pub from: User,
    pub invoice_payload: CompactString,
    pub shipping_address: ShippingAddress,
}

/// This object represents a shipping address.
/// https://core.telegram.org/bots/api#shippingaddress
#[derive(Debug, Deserialize, Serialize)]
pub struct ShippingAddress {
    pub country_code: CompactString,
    pub state: CompactString,
    pub city: CompactString,
    pub street_line1: CompactString,
    pub street_line2: CompactString,
    pub post_code: CompactString,
}

/// This object contains information about an incoming pre-checkout query.
/// https://core.telegram.org/bots/api#precheckoutquery
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct PreCheckoutQuery {
    pub id: CompactString,
    pub from: User,
    pub currency: CompactString,
    pub total_amount: u64,
    pub invoice_payload: CompactString,
    pub shipping_option_id: Option<CompactString>,
    pub order_info: Option<OrderInfo>,
}

/// This object represents information about an order.
/// https://core.telegram.org/bots/api#orderinfo
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct OrderInfo {
    pub name: Option<CompactString>,
    pub phone_number: Option<CompactString>,
    pub email: Option<CompactString>,
    pub shipping_address: Option<ShippingAddress>,
}

/// This object represents an answer of a user in a non-anonymous poll.
/// https://core.telegram.org/bots/api#pollanswer
#[derive(Debug, Deserialize, Serialize)]
pub struct PollAnswer {
    poll_id: CompactString,
    user: User,
    option_ids: Vec<u64>,
}

/// This object represents changes in the status of a [chat member](https://core.telegram.org/bots/api#chatmember).
/// https://core.telegram.org/bots/api#chatmemberupdated
#[derive(Debug, Deserialize)]
pub struct ChatMemberUpdated {
    pub chat: Chat,
    pub from: User,
    pub date: Date,
    pub old_chat_member: ChatMember,
    pub new_chat_member: ChatMember,
    pub invite_link: Option<ChatInviteLink>,
}

/// This object contains information about one member of a chat.
/// Currently, the following 6 types of chat members are supported:
/// - [ChatMemberOwner](https://core.telegram.org/bots/api#chatmemberowner)
/// - [ChatMemberAdministrator](https://core.telegram.org/bots/api#chatmemberadministrator)
/// - [ChatMemberMember](https://core.telegram.org/bots/api#chatmembermember)
/// - [ChatMemberRestricted](https://core.telegram.org/bots/api#chatmemberrestricted)
/// - [ChatMemberLeft](https://core.telegram.org/bots/api#chatmemberleft)
/// - [ChatMemberBanned](https://core.telegram.org/bots/api#chatmemberbanned)
/// https://core.telegram.org/bots/api#chatmember
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub enum ChatMember {
    #[serde(rename = "creator")]
    ChatMemberOwner(ChatMemberOwner),
    #[serde(rename = "administrator")]
    ChatMemberAdministrator(ChatMemberAdministrator),
    #[serde(rename = "member")]
    ChatMemberMember(ChatMemberMember),
    #[serde(rename = "restricted")]
    ChatMemberRestricted(ChatMemberRestricted),
    #[serde(rename = "left")]
    ChatMemberLeft(ChatMemberLeft),
    #[serde(rename = "kicked")]
    ChatMemberBanned(ChatMemberBanned),
}

/// Represents a [chat member](https://core.telegram.org/bots/api#chatmember) that owns the chat and has all administrator privileges.
/// https://core.telegram.org/bots/api#chatmemberowner
#[derive(Debug, Deserialize)]
pub struct ChatMemberOwner {
    pub user: User,
    pub is_anonymous: bool,
    pub custom_title: Option<CompactString>,
}

/// Represents a [chat member](https://core.telegram.org/bots/api#chatmember) that has some additional privileges.
/// https://core.telegram.org/bots/api#chatmemberadministrator
#[derive(Debug, Deserialize)]
pub struct ChatMemberAdministrator {
    pub user: User,
    pub can_be_edited: bool,
    pub is_anonymous: bool,
    pub can_manage_chat: bool,
    pub can_delete_messages: bool,
    pub can_manage_video_chats: bool,
    pub can_restrict_members: bool,
    pub can_promote_members: bool,
    pub can_change_info: bool,
    pub can_invite_users: bool,
    pub can_post_messages: Option<bool>,
    pub can_edit_messages: Option<bool>,
    pub can_pin_messages: Option<bool>,
    pub can_manage_topics: Option<bool>,
    pub custom_title: Option<CompactString>,
}

/// Represents a [chat member](https://core.telegram.org/bots/api#chatmember) that has no additional privileges or restrictions.
/// https://core.telegram.org/bots/api#chatmembermember
#[derive(Debug, Deserialize)]
pub struct ChatMemberMember {
    pub user: User,
}

/// Represents a [chat member](https://core.telegram.org/bots/api#chatmember) that is under certain restrictions in the chat. Supergroups only.
/// https://core.telegram.org/bots/api#chatmemberrestricted
#[derive(Debug, Deserialize)]
pub struct ChatMemberRestricted {
    pub user: User,
    pub is_member: bool,
    pub can_send_messages: bool,
    pub can_send_audios: bool,
    pub can_send_documents: bool,
    pub can_send_photos: bool,
    pub can_send_videos: bool,
    pub can_send_video_notes: bool,
    pub can_send_voice_notes: bool,
    pub can_send_polls: bool,
    pub can_send_other_messages: bool,
    pub can_add_web_page_previews: bool,
    pub can_change_info: bool,
    pub can_invite_users: bool,
    pub can_pin_messages: bool,
    pub can_manage_topics: bool,
    pub until_date: Date,
}

/// Represents a [chat member](https://core.telegram.org/bots/api#chatmember) that isn't currently a member of the chat, but may join it themselves.
/// https://core.telegram.org/bots/api#chatmemberleft
#[derive(Debug, Deserialize)]
pub struct ChatMemberLeft {
    pub user: User,
}

/// Represents a [chat member](https://core.telegram.org/bots/api#chatmember) that was banned in the chat and can't return to the chat or view chat messages.
/// https://core.telegram.org/bots/api#chatmemberbanned
#[derive(Debug, Deserialize)]
pub struct ChatMemberBanned {
    pub user: User,
    pub until_date: Date,
}

/*impl<'de> Deserialize<'de> for ChatMember {
    fn deserialize<D>(deserializer: D) -> Result<ChatMember, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = Map::deserialize(deserializer)?;

        let status = map
            .remove("status")
            .ok_or_else(|| de::Error::missing_field("status"))
            .map(CompactString::deserialize)?
            .map_err(de::Error::custom)?;

        let obj = Value::Object(map);
        if status == "creator" {
            let member = ChatMemberOwner::deserialize(obj).map_err(de::Error::custom)?;
            Ok(ChatMember::ChatMemberOwner(member))
        } else if status == "administrator" {
            let member = ChatMemberAdministrator::deserialize(obj).map_err(de::Error::custom)?;
            Ok(ChatMember::ChatMemberAdministrator(member))
        } else if status == "member" {
            let member = ChatMemberMember::deserialize(obj).map_err(de::Error::custom)?;
            Ok(ChatMember::ChatMemberMember(member))
        } else if status == "restricted" {
            let member = ChatMemberRestricted::deserialize(obj).map_err(de::Error::custom)?;
            Ok(ChatMember::ChatMemberRestricted(member))
        } else if status == "left" {
            let member = ChatMemberLeft::deserialize(obj).map_err(de::Error::custom)?;
            Ok(ChatMember::ChatMemberLeft(member))
        } else if status == "kicked" {
            let member = ChatMemberBanned::deserialize(obj).map_err(de::Error::custom)?;
            Ok(ChatMember::ChatMemberBanned(member))
        } else {
            Err(de::Error::custom(
                "unknown [chat member](https://core.telegram.org/bots/api#chatmember)",
            ))
        }
    }
}*/

/// Represents an invite link for a chat.
/// https://core.telegram.org/bots/api#chatinvitelink
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatInviteLink {
    pub invite_link: CompactString,
    pub creator: User,
    pub creates_join_request: bool,
    pub is_primary: bool,
    pub is_revoked: bool,
    pub name: Option<CompactString>,
    pub expire_date: Option<Date>,
    pub member_limit: Option<u32>,
    pub pending_join_request_count: Option<u64>,
}

/// Represents a join request sent to a chat.
/// https://core.telegram.org/bots/api#chatjoinrequest
#[derive(Debug, Deserialize)]
pub struct ChatJoinRequest {
    pub chat: Chat,
    pub from: User,
    pub user_chat_id: i64,
    pub date: Date,
    pub bio: Option<CompactString>,
    pub invite_link: Option<ChatInviteLink>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChatId {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    Int(ChatIntId),
    Str(CompactString),
}

impl From<i64> for ChatId {
    fn from(id: i64) -> Self {
        Self::Int(id)
    }
}

impl From<CompactString> for ChatId {
    fn from(id: CompactString) -> Self {
        Self::Str(id)
    }
}

impl Default for ChatId {
    fn default() -> Self {
        // hi
        ChatId::from(-1001738773095)
    }
}

impl From<&ChatId> for ChatIntId {
    fn from(value: &ChatId) -> Self {
        match value {
            ChatId::Int(id) => *id,
            ChatId::Str(id) => {
                let Some(id) = id.strip_prefix('@') else {
                    panic!("chat id '{id}' did not contain '@'");
                };
                id.parse::<ChatIntId>().unwrap_or_else(|err| {
                    panic!("failed to convert chat id '{id}' to integer, {err}")
                })
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ParseMode {
    #[serde(rename = "HTML")]
    Html,
    Markdown,
    MarkdownV2,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ReplyMarkup {
    InlineKeyboardMarkup(InlineKeyboardMarkup),
    ReplyKeyboardMarkup(ReplyKeyboardMarkup),
    ReplyKeyboardRemove(ReplyKeyboardRemove),
    ForceReply(ForceReply),
}

/// This object represents a [custom keyboard](https://core.telegram.org/bots/features#keyboards)
/// with reply options (see [Introduction to bots](https://core.telegram.org/bots/features#keyboards) for details and examples).
/// https://core.telegram.org/bots/api#replykeyboardmarkup
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<KeyboardButton>>,
    pub is_persistent: Option<bool>,
    pub resize_keyboard: Option<bool>,
    pub one_time_keyboard: Option<bool>,
    pub input_field_placeholder: Option<CompactString>,
    pub selective: Option<bool>,
}

/// This object represents one button of the reply keyboard.
/// For simple text buttons, String can be used instead of this object to specify the button text.
/// The optional fields `web_app`, `request_user`, `request_chat`, `request_contact`,
/// `request_location`, and `request_poll` are mutually exclusive.
/// https://core.telegram.org/bots/api#keyboardbutton
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct KeyboardButton {
    pub text: CompactString,
    pub request_user: Option<KeyboardButtonRequestUser>,
    pub request_chat: Option<KeyboardButtonRequestChat>,
    pub request_contact: Option<bool>,
    pub request_location: Option<bool>,
    pub request_poll: Option<KeyboardButtonPollType>,
    pub web_app: Option<WebAppInfo>,
}

/// This object defines the criteria used to request a suitable user.
/// The identifier of the selected user will be shared with the bot when the corresponding button is pressed.
/// https://core.telegram.org/bots/api#keyboardbuttonrequestuser
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct KeyboardButtonRequestUser {
    pub request_id: i32,
    pub user_is_bot: Option<bool>,
    pub user_is_premium: Option<bool>,
}

/// This object defines the criteria used to request a suitable chat.
/// The identifier of the selected chat will be shared with the bot when the corresponding button is pressed.
/// https://core.telegram.org/bots/api#keyboardbuttonrequestchat
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct KeyboardButtonRequestChat {
    pub request_id: i32,
    pub chat_is_channel: Option<bool>,
    pub chat_is_forum: Option<bool>,
    pub chat_has_username: Option<bool>,
    pub chat_is_created: Option<bool>,
    pub user_administrator_rights: Option<ChatAdministratorRights>,
    pub bot_administrator_rights: Option<ChatAdministratorRights>,
    pub bot_is_member: Option<bool>,
}

/// Represents the rights of an administrator in a chat.
/// https://core.telegram.org/bots/api#chatadministratorrights
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatAdministratorRights {
    pub is_anonymous: bool,
    pub can_manage_chat: bool,
    pub can_delete_messages: bool,
    pub can_manage_video_chats: bool,
    pub can_restrict_members: bool,
    pub can_promote_members: bool,
    pub can_change_info: bool,
    pub can_invite_users: bool,
    pub can_post_messages: Option<bool>,
    pub can_edit_messages: Option<bool>,
    pub can_pin_messages: Option<bool>,
    pub can_manage_topics: Option<bool>,
}

/// This object represents type of a poll, which is allowed to be created and sent when the corresponding button is pressed.
/// https://core.telegram.org/bots/api#keyboardbuttonpolltype
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct KeyboardButtonPollType {
    #[serde(default, rename = "type")]
    poll_type: Option<PollType>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    Quiz,
    Regular,
}

/// Describes a [Web App](https://core.telegram.org/bots/webapps).
/// https://core.telegram.org/bots/webapps
#[derive(Debug, Deserialize, Serialize)]
pub struct WebAppInfo {
    pub url: CompactString,
}

/// Upon receiving a message with this object,
/// Telegram clients will remove the current custom keyboard and display the default letter-keyboard.
/// By default, custom keyboards are displayed until a new keyboard is sent by a bot.
/// An exception is made for one-time keyboards that are hidden immediately after
/// the user presses a button (see [ReplyKeyboardMarkup](https://core.telegram.org/bots/api#replykeyboardmarkup)).
/// https://core.telegram.org/bots/api#replykeyboardremove
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyKeyboardRemove {
    pub remove_keyboard: bool,
    pub selective: Option<bool>,
}

/// Upon receiving a message with this object,
/// Telegram clients will display a reply interface to the user
/// (act as if the user has selected the bot's message and tapped 'Reply').
/// This can be extremely useful if you want to create user-friendly step-by-step interfaces
/// without having to sacrifice [privacy mode](https://core.telegram.org/bots/features#privacy-mode).
/// https://core.telegram.org/bots/api#forcereply
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ForceReply {
    pub force_reply: bool,
    pub input_field_placeholder: Option<CompactString>,
    pub selective: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    #[serde(default)]
    pub is_bot: bool,
    pub first_name: CompactString,
    pub last_name: Option<CompactString>,
    pub username: Option<CompactString>,
    pub language_code: Option<CompactString>,
    pub is_premium: Option<bool>,
    pub added_to_attachment_menu: Option<bool>,
    pub can_join_groups: Option<bool>,
    pub can_read_all_group_messages: Option<bool>,
    pub supports_inline_queries: Option<bool>,
}

impl User {
    pub fn full_name(&self) -> CompactString {
        let mut name = self.first_name.clone();
        if let Some(s) = &self.last_name {
            name += s.as_str();
        }
        name
    }

    pub fn full_name_with_username(&self) -> CompactString {
        let mut name = self.first_name.clone();
        if let Some(s) = &self.username {
            name += &format!(" {s}");
        }
        if let Some(s) = &self.last_name {
            name += &format!(" {s}");
        }
        name
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Sender,
    #[default]
    Private,
    Group,
    Supergroup,
    Channel,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: ChatIntId,
    #[serde(default, rename = "type")]
    pub chat_type: ChatType,
    pub title: Option<CompactString>,
    pub username: Option<CompactString>,
    pub first_name: Option<CompactString>,
    pub last_name: Option<CompactString>,
    pub is_forum: Option<bool>,
    pub photo: Option<ChatPhoto>,
    pub active_usernames: Option<Vec<CompactString>>,
    pub emoji_status_custom_emoji_id: Option<CompactString>,
    pub bio: Option<CompactString>,
    pub has_private_forwards: Option<bool>,
    pub has_restricted_voice_and_video_messages: Option<bool>,
    pub join_by_request: Option<bool>,
    pub description: Option<CompactString>,
    pub invite_link: Option<CompactString>,
    pub pinned_message: Option<Box<Message>>,
    pub permissions: Option<ChatPermissions>,
    pub slow_mode_delay: Option<i64>,
    pub message_auto_delete_time: Option<i64>,
    pub has_aggressive_anti_spam_enabled: Option<bool>,
    pub has_hidden_members: Option<bool>,
    pub has_protected_content: Option<bool>,
    pub sticker_set_name: Option<CompactString>,
    pub can_set_sticker_set: Option<bool>,
    pub linked_chat_id: Option<i64>,
    pub location: Option<ChatLocation>,
}

impl Chat {
    pub fn title_with_full_name(&self) -> Option<CompactString> {
        let mut name: Option<CompactString> = None;
        if let Some(s) = &self.title {
            name.replace(s.clone());
        }
        if let Some(s) = &self.first_name {
            match &mut name {
                Some(n) => *n += &format!(" {s}"),
                None => {
                    name.replace(s.clone());
                }
            }
        }
        if let Some(s) = &self.username {
            match &mut name {
                Some(n) => *n += &format!(" {s}"),
                None => {
                    name.replace(s.clone());
                }
            }
        }
        if let Some(s) = &self.last_name {
            match &mut name {
                Some(n) => *n += &format!(" {s}"),
                None => {
                    name.replace(s.clone());
                }
            }
        }
        name
    }
}

impl Default for Chat {
    fn default() -> Self {
        Chat {
            id: 0,
            chat_type: ChatType::Private,
            title: None,
            username: None,
            first_name: None,
            last_name: None,
            is_forum: None,
            photo: None,
            active_usernames: None,
            emoji_status_custom_emoji_id: None,
            bio: None,
            has_private_forwards: None,
            has_restricted_voice_and_video_messages: None,
            join_by_request: None,
            description: None,
            invite_link: None,
            pinned_message: None,
            permissions: None,
            slow_mode_delay: None,
            message_auto_delete_time: None,
            has_aggressive_anti_spam_enabled: None,
            has_hidden_members: None,
            has_protected_content: None,
            sticker_set_name: None,
            can_set_sticker_set: None,
            linked_chat_id: None,
            location: None,
        }
    }
}

/// Represents a location to which a chat is connected.
/// https://core.telegram.org/bots/api#chatlocation
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatLocation {
    pub location: Location,
    pub address: CompactString,
}

/// Describes actions that a non-administrator user is allowed to take in a chat.
/// https://core.telegram.org/bots/api#chatpermissions
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatPermissions {
    pub can_send_messages: Option<bool>,
    pub can_send_audios: Option<bool>,
    pub can_send_documents: Option<bool>,
    pub can_send_photos: Option<bool>,
    pub can_send_videos: Option<bool>,
    pub can_send_video_notes: Option<bool>,
    pub can_send_voice_notes: Option<bool>,
    pub can_send_polls: Option<bool>,
    pub can_send_other_messages: Option<bool>,
    pub can_add_web_page_previews: Option<bool>,
    pub can_change_info: Option<bool>,
    pub can_invite_users: Option<bool>,
    pub can_pin_messages: Option<bool>,
    pub can_manage_topics: Option<bool>,
}

/// This object represents a chat photo.
/// https://core.telegram.org/bots/api#chatphoto
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatPhoto {
    pub small_file_id: CompactString,
    pub small_file_unique_id: CompactString,
    pub big_file_id: CompactString,
    pub big_file_unique_id: CompactString,
}

/// This object represents one special entity in a text message. For example, hashtags, usernames, URLs, etc.
/// https://core.telegram.org/bots/api#messageentity
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageEntity {
    #[serde(rename = "type")]
    pub entity_type: MessageEntityType,
    pub offset: i64,
    pub length: usize,
    pub url: Option<CompactString>,
    pub user: Option<User>,
    pub language: Option<CompactString>,
    pub custom_emoji_id: Option<CompactString>,
}

/// Type of the entity. Currently, can be “mention” (@username), “hashtag” (#hashtag),
/// “cashtag” ($USD), “bot_command” (/start@jobs_bot), “url” (https://telegram.org),
/// “email” (do-not-reply@telegram.org), “phone_number” (+1-212-555-0123), “bold” (bold text),
/// “italic” (italic text), “underline” (underlined text), “strikethrough” (strikethrough text),
/// “spoiler” (spoiler message), “code” (monowidth string), “pre” (monowidth block),
/// “text_link” (for clickable text URLs), “text_mention” (for users
/// [without usernames](https://telegram.org/blog/edit#new-mentions)),
/// “custom_emoji” (for inline custom emoji stickers)
/// https://core.telegram.org/bots/api#messageentity
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageEntityType {
    Mention,
    Hashtag,
    Cashtag,
    BotCommand,
    Url,
    Email,
    PhoneNumber,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Spoiler,
    Code,
    Pre,
    TextLink,
    TextMention,
    CustomEmoji,
}

/// This object represents an animation file (GIF or H.264/MPEG-4 AVC video without sound).
/// https://core.telegram.org/bots/api#animation
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Animation {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    pub thumbnail: Option<PhotoSize>,
    pub file_name: Option<CompactString>,
    pub mime_type: Option<CompactString>,
    pub file_size: Option<i64>,
}

/// This object represents an audio file to be treated as music by the Telegram clients.
/// https://core.telegram.org/bots/api#audio
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Audio {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub duration: i32,
    pub performer: Option<CompactString>,
    pub title: Option<CompactString>,
    pub file_name: Option<CompactString>,
    pub mime_type: Option<CompactString>,
    pub file_size: Option<i64>,
    pub thumbnail: Option<PhotoSize>,
}

/// This object represents a general file (as opposed to photos, voice messages and audio files).
/// https://core.telegram.org/bots/api#document
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Document {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub thumbnail: Option<PhotoSize>,
    pub file_name: Option<CompactString>,
    pub mime_type: Option<CompactString>,
    pub file_size: Option<i64>,
}

/// This object represents one size of a photo
/// or a [file](https://core.telegram.org/bots/api#document) /
/// [sticker](https://core.telegram.org/bots/api#sticker) thumbnail.
/// https://core.telegram.org/bots/api#photosize
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct PhotoSize {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub width: i32,
    pub height: i32,
    pub file_size: Option<i64>,
}

/// This object represents a sticker.
/// https://core.telegram.org/bots/api#sticker
#[derive(Debug, Deserialize)]
pub struct Sticker {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    #[serde(rename = "type")]
    pub sticker_type: StickerType,
    pub width: i32,
    pub height: i32,
    pub is_animated: bool,
    pub is_video: bool,
    pub thumbnail: Option<PhotoSize>,
    pub emoji: Option<CompactString>,
    pub set_name: Option<CompactString>,
    pub premium_animation: Option<File>,
    pub mask_position: Option<MaskPosition>,
    pub custom_emoji_id: Option<CompactString>,
    pub needs_repainting: Option<bool>,
    pub file_size: Option<i64>,
}

/// This object describes the position on faces where a mask should be placed by default.
/// https://core.telegram.org/bots/api#maskposition
#[derive(Debug, Deserialize)]
pub struct MaskPosition {
    pub point: CompactString,
    pub x_shift: f32,
    pub y_shift: f32,
    pub scale: f32,
}

/// This object represents a file ready to be downloaded.
/// The file can be downloaded via the link `https://api.telegram.org/file/bot<token>/<file_path>`.
/// It is guaranteed that the link will be valid for at least 1 hour.
/// When the link expires, a new one can be requested
/// by calling [getFile](https://core.telegram.org/bots/api#getfile).
/// https://core.telegram.org/bots/api#file
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub file_size: Option<i64>,
    pub file_path: Option<CompactString>,
}

/// Type of the sticker, currently one of “regular”, “mask”, “custom_emoji”.
/// The type of the sticker is independent from its format,
/// which is determined by the fields `is_animated` and `is_video`.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StickerType {
    Regular,
    Mask,
    CustomEmoji,
}

/// This object represents a video file.
/// https://core.telegram.org/bots/api#video
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Video {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    pub thumbnail: Option<PhotoSize>,
    pub file_name: Option<CompactString>,
    pub mime_type: Option<CompactString>,
    pub file_size: Option<i64>,
}

/// This object represents a [video message](https://telegram.org/blog/video-messages-and-telescope)
/// (available in Telegram apps as of [v.4.0](https://telegram.org/blog/video-messages-and-telescope)).
/// https://core.telegram.org/bots/api#videonote
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoNote {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub length: i32,
    pub duration: i32,
    pub thumbnail: Option<PhotoSize>,
    pub file_size: Option<i64>,
}

/// This object represents a voice note.
/// https://core.telegram.org/bots/api#voice
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Voice {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub duration: i32,
    pub mime_type: Option<CompactString>,
    pub file_size: Option<i64>,
}

/// This object represents an animated emoji that displays a random value.
/// https://core.telegram.org/bots/api#dice
#[derive(Debug, Deserialize, Serialize)]
pub struct Dice {
    pub emoji: CompactString,
    pub value: u8,
}

/// This object represents a game. Use BotFather to create and edit games, their short names will act as unique identifiers.
/// https://core.telegram.org/bots/api#game
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Game {
    pub title: CompactString,
    pub description: CompactString,
    pub photo: Vec<PhotoSize>,
    pub text: Option<CompactString>,
    pub text_entities: Option<Vec<MessageEntity>>,
    pub animation: Option<Animation>,
}

/// This object contains information about a poll.
/// https://core.telegram.org/bots/api#poll
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Poll {
    pub id: CompactString,
    pub question: CompactString,
    pub options: Vec<PollOption>,
    pub total_voter_count: i32,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub poll_type: PollType,
    pub allows_multiple_answers: bool,
    pub correct_option_id: Option<i32>,
    pub explanation: Option<CompactString>,
    pub explanation_entities: Option<Vec<MessageEntity>>,
    pub open_period: Option<i32>,
    pub close_date: Option<Timestamp>,
}

/// This object contains information about one answer option in a poll.
/// https://core.telegram.org/bots/api#polloption
#[derive(Debug, Deserialize, Serialize)]
pub struct PollOption {
    text: CompactString,
    voter_count: i32,
}

/// This object represents a venue.
/// https://core.telegram.org/bots/api#venue
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Venue {
    pub location: Location,
    pub title: CompactString,
    pub address: CompactString,
    pub foursquare_id: Option<CompactString>,
    pub foursquare_t: Option<CompactString>,
    pub google_place_id: Option<CompactString>,
    pub google_place_type: Option<CompactString>,
}

/// This object represents a service message about a change in auto-delete timer settings.
/// https://core.telegram.org/bots/api#messageautodeletetimerchanged
#[derive(Debug, Deserialize, Serialize)]
pub struct MessageAutoDeleteTimerChanged {
    pub message_auto_delete_time: i32,
}

/// This object represents a point on the map.
/// https://core.telegram.org/bots/api#location
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pub longitude: f32,
    pub latitude: f32,
    pub horizontal_accuracy: Option<f32>,
    pub live_period: Option<i32>,
    pub heading: Option<i16>,
    pub proximity_alert_radius: Option<i32>,
}

/// This object contains basic information about an invoice.
/// https://core.telegram.org/bots/api#invoice
#[derive(Debug, Deserialize, Serialize)]
pub struct Invoice {
    pub title: CompactString,
    pub description: CompactString,
    pub start_parameter: CompactString,
    pub currency: CompactString,
    pub total_amount: i32,
}

/// This object contains basic information about a successful payment.
/// https://core.telegram.org/bots/api#successfulpayment
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessfulPayment {
    pub currency: CompactString,
    pub total_amount: i32,
    pub invoice_payload: CompactString,
    pub shipping_option_id: Option<CompactString>,
    pub order_info: Option<OrderInfo>,
    pub telegram_payment_charge_id: CompactString,
    pub provider_payment_charge_id: CompactString,
}

/// This object contains information about the user whose identifier was shared with the bot
/// using a [KeyboardButtonRequestUser](https://core.telegram.org/bots/api#keyboardbuttonrequestuser) button.
/// https://core.telegram.org/bots/api#usershared
#[derive(Debug, Deserialize, Serialize)]
pub struct UserShared {
    pub request_id: i32,
    pub user_id: UserId,
}

/// This object contains information about the chat whose identifier was shared with the bot
/// using a [KeyboardButtonRequestChat](https://core.telegram.org/bots/api#keyboardbuttonrequestchat) button.
/// https://core.telegram.org/bots/api#chatshared
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatShared {
    pub request_id: i32,
    pub chat_id: i64,
}

/// This object represents a service message about a user allowing a bot added to the attachment menu to write messages.
/// Currently holds no information.
/// https://core.telegram.org/bots/api#writeaccessallowed
#[derive(Debug, Deserialize, Serialize)]
pub struct WriteAccessAllowed;

/// Describes Telegram Passport data shared with the bot by the user.
/// https://core.telegram.org/bots/api#passportdata
#[derive(Debug, Deserialize, Serialize)]
pub struct PassportData {
    pub data: Vec<EncryptedPassportElement>,
    pub credentials: EncryptedCredentials,
}

/// Describes documents or other Telegram Passport elements shared with the bot by the user.
/// https://core.telegram.org/bots/api#encryptedpassportelement
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct EncryptedPassportElement {
    pub element_type: PassportElementType,
    pub data: Option<CompactString>,
    pub phone_number: Option<CompactString>,
    pub email: Option<CompactString>,
    pub files: Option<Vec<PassportFile>>,
    pub front_side: Option<PassportFile>,
    pub reverse_side: Option<PassportFile>,
    pub selfie: Option<PassportFile>,
    pub translation: Option<Vec<PassportFile>>,
    pub hash: CompactString,
}

/// This object represents a file uploaded to Telegram Passport.
/// Currently all Telegram Passport files are in JPEG format when decrypted and don't exceed 10MB.
/// https://core.telegram.org/bots/api#passportfile
#[derive(Debug, Deserialize, Serialize)]
pub struct PassportFile {
    pub file_id: CompactString,
    pub file_unique_id: CompactString,
    pub file_size: CompactString,
    pub file_date: Timestamp,
}

/// Element type. One of “personal_details”, “passport”, “driver_license”, “identity_card”,
/// “internal_passport”, “address”, “utility_bill”, “bank_statement”, “rental_agreement”,
/// “passport_registration”, “temporary_registration”, “phone_number”, “email”.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PassportElementType {
    PersonalDetails,
    Passport,
    DriverLicense,
    IdentityCard,
    InternalPassport,
    Address,
    UtilityBill,
    BankStatement,
    RentalAgreement,
    PassportRegistration,
    TemporaryRegistration,
    PhoneNumber,
    Email,
}

/// Describes documents or other Telegram Passport elements shared with the bot by the user.
/// https://core.telegram.org/bots/api#encryptedpassportelement
#[derive(Debug, Deserialize, Serialize)]
pub struct EncryptedCredentials {
    pub data: CompactString,
    pub hash: CompactString,
    pub secret: CompactString,
}

/// https://core.telegram.org/bots/api#proximityalerttriggered
/// This object represents the content of a service message,
/// sent whenever a user in the chat triggers a proximity alert set by another user.
#[derive(Debug, Deserialize, Serialize)]
pub struct ProximityAlertTriggered {
    pub traveler: User,
    pub watcher: User,
    pub distance: i32,
}

/// This object represents a service message about a new forum topic created in the chat.
/// https://core.telegram.org/bots/api#forumtopiccreated
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ForumTopicCreated {
    pub name: CompactString,
    pub icon_color: i32,
    pub icon_custom_emoji_id: Option<CompactString>,
}

/// This object represents a service message about an edited forum topic.
/// https://core.telegram.org/bots/api#forumtopicedited
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ForumTopicEdited {
    pub name: Option<CompactString>,
    pub icon_color: i32,
    pub icon_custom_emoji_id: Option<CompactString>,
}

/// This object represents a service message about a forum topic closed in the chat.
/// Currently holds no information.
/// https://core.telegram.org/bots/api#forumtopicclosed
#[derive(Debug, Deserialize, Serialize)]
pub struct ForumTopicClosed;

/// This object represents a service message about a forum topic reopened in the chat.
/// Currently holds no information.
#[derive(Debug, Deserialize, Serialize)]
pub struct ForumTopicReopened;

/// This object represents a service message about General forum topic hidden in the chat.
/// Currently holds no information.
/// https://core.telegram.org/bots/api#forumtopichidden
#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralForumTopicHidden;

/// This object represents a service message about General forum topic unhidden in the chat.
/// Currently holds no information.
/// https://core.telegram.org/bots/api#forumtopicunhidden
#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralForumTopicUnhidden;

/// This object represents a service message about a video chat scheduled in the chat.
/// https://core.telegram.org/bots/api#videochatscheduled
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoChatScheduled {
    pub start_date: Timestamp,
}

/// This object represents a service message about a video chat started in the chat.
/// Currently holds no information.
/// https://core.telegram.org/bots/api#videochatstarted
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoChatStarted;

/// This object represents a service message about a video chat ended in the chat.
/// https://core.telegram.org/bots/api#videochatended
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoChatEnded {
    pub duration: i32,
}

/// This object represents a service message about new members invited to a video chat.
/// https://core.telegram.org/bots/api#videochatparticipantsinvited
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoChatParticipantsInvited {
    pub users: Vec<User>,
}

/// Describes data sent from a [Web App](https://core.telegram.org/bots/webapps) to the bot.
/// https://core.telegram.org/bots/api#webappdata
#[derive(Debug, Deserialize, Serialize)]
pub struct WebAppData {
    pub data: CompactString,
    pub button_text: CompactString,
}

/// This object represents an [inline keyboard](https://core.telegram.org/bots/features#inline-keyboards)
/// that appears right next to the message it belongs to.
/// https://core.telegram.org/bots/api#inlinekeyboardmarkup
#[derive(Debug, Deserialize, Serialize)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

/// This object represents one button of an inline keyboard.
/// You **must** use exactly one of the optional fields.
/// https://core.telegram.org/bots/api#inlinekeyboardbutton
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct InlineKeyboardButton {
    pub text: CompactString,
    pub url: Option<CompactString>,
    pub callback_data: Option<CompactString>,
    pub web_app: Option<WebAppInfo>,
    pub login_url: Option<LoginUrl>,
    pub switch_inline_query: Option<CompactString>,
    pub switch_inline_query_current_chat: Option<CompactString>,
    pub callback_game: Option<CallbackGame>,
    pub pay: Option<bool>,
}

/// This object represents a parameter of the inline keyboard button used to automatically authorize a user.
/// Serves as a great replacement for the Telegram Login Widget when the user is coming from Telegram.
/// All the user needs to do is tap/click a button and confirm that they want to log in.
/// https://core.telegram.org/bots/api#loginurl
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUrl {
    pub url: CompactString,
    pub forward_text: Option<CompactString>,
    pub bot_username: Option<CompactString>,
    pub request_write_access: Option<bool>,
}

/// A placeholder, currently holds no information. Use BotFather to set up your game.
/// https://core.telegram.org/bots/api#callbackgame
#[derive(Debug, Deserialize, Serialize)]
pub struct CallbackGame {}

/// This object represents a phone contact.
/// https://core.telegram.org/bots/api#contact
#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub phone_number: CompactString,
    pub first_name: CompactString,
    pub last_name: Option<CompactString>,
    pub user_id: Option<UserId>,
    pub vcard: Option<CompactString>,
}

pub static DELETED_ACCOUNT: &str = "Deleted Account";

#[derive(Debug, Default, Deserialize)]
pub struct Message {
    pub message_id: MessageId,
    pub message_thread_id: Option<i64>,
    pub from: Option<User>,
    pub sender_chat: Option<Chat>,
    pub date: Date,
    pub chat: Chat,
    pub forward_from: Option<User>,
    pub forward_from_chat: Option<Chat>,
    pub forward_from_message_id: Option<MessageId>,
    pub forward_signature: Option<CompactString>,
    pub forward_sender_name: Option<CompactString>,
    pub forward_date: Option<i64>,
    pub is_topic_message: Option<bool>,
    pub is_automatic_forward: Option<bool>,
    pub reply_to_message: Option<Box<Message>>,
    pub via_bot: Option<User>,
    pub edit_date: Option<i64>,
    pub has_protected_content: Option<bool>,
    pub media_group_id: Option<CompactString>,
    pub author_signature: Option<CompactString>,
    pub text: Option<CompactString>,
    pub entities: Option<Vec<MessageEntity>>,
    pub animation: Option<Animation>,
    pub audio: Option<Audio>,
    pub document: Option<Document>,
    pub photo: Option<Box<Vec<PhotoSize>>>,
    pub sticker: Option<Sticker>,
    pub video: Option<Video>,
    pub video_note: Option<VideoNote>,
    pub voice: Option<Voice>,
    pub caption: Option<CompactString>,
    pub caption_entities: Option<Vec<MessageEntity>>,
    pub has_media_spoiler: Option<bool>,
    pub contact: Option<Contact>,
    pub dice: Option<Dice>,
    pub game: Option<Game>,
    pub poll: Option<Poll>,
    pub venue: Option<Venue>,
    pub location: Option<Location>,
    pub new_chat_members: Option<Box<Vec<User>>>,
    pub left_chat_member: Option<User>,
    pub new_chat_title: Option<CompactString>,
    pub new_chat_photo: Option<Box<Vec<PhotoSize>>>,
    pub delete_chat_photo: Option<bool>,
    pub group_chat_created: Option<bool>,
    pub supergroup_chat_created: Option<bool>,
    pub channel_chat_created: Option<bool>,
    pub message_auto_delete_timer_changed: Option<MessageAutoDeleteTimerChanged>,
    pub migrate_to_chat_id: Option<i64>,
    pub migrate_from_chat_id: Option<i64>,
    pub pinned_message: Option<Box<Message>>,
    pub invoice: Option<Invoice>,
    pub successful_payment: Option<SuccessfulPayment>,
    pub user_shared: Option<UserShared>,
    pub chat_shared: Option<ChatShared>,
    pub connected_website: Option<CompactString>,
    pub write_access_allowed: Option<WriteAccessAllowed>,
    pub passport_data: Option<PassportData>,
    pub proximity_alert_triggered: Option<ProximityAlertTriggered>,
    pub forum_topic_created: Option<ForumTopicCreated>,
    pub forum_topic_edited: Option<ForumTopicEdited>,
    pub forum_topic_closed: Option<ForumTopicClosed>,
    pub forum_topic_reopened: Option<ForumTopicReopened>,
    pub general_forum_topic_hidden: Option<GeneralForumTopicHidden>,
    pub general_forum_topic_unhidden: Option<GeneralForumTopicUnhidden>,
    pub video_chat_scheduled: Option<VideoChatScheduled>,
    pub video_chat_started: Option<VideoChatStarted>,
    pub video_chat_ended: Option<VideoChatEnded>,
    pub video_chat_participants_invited: Option<VideoChatParticipantsInvited>,
    pub web_app_data: Option<WebAppData>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
}

impl Message {
    pub fn is_of_entity(&self, entity: MessageEntityType) -> Option<MessageEntity> {
        if let Some(entities) = &self.entities {
            for msg_entity in entities {
                if msg_entity.entity_type == entity {
                    return Some(msg_entity.clone());
                }
            }
        }
        None
    }

    pub fn is_forwarded_from_deleted_account(&self) -> bool {
        match self.forward_sender_name.as_ref() {
            None => false,
            Some(name) => name.as_str() == DELETED_ACCOUNT,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatAction {
    Typing,
    UploadPhoto,
    RecordVideo,
    UploadVideo,
    RecordVoice,
    UploadVoice,
    UploadDocument,
    ChooseSticker,
    FindLocation,
    RecordVideoNote,
    UploadVideoNote,
}

#[cfg(test)]
mod tests {
    use crate::proto::CommonUpdate;

    #[test]
    fn deserialize_common_update() {
        let data = serde_json::json!({
            "message": {
                "chat": {
                    "first_name": "Test",
                    "id": 1111111,
                    "last_name": "Test Lastname",
                    "username": "Test"
                },
                "date": 1441645532,
                "from": {
                    "first_name": "Test",
                    "id": 1111111,
                    "last_name": "Test Lastname",
                    "username": "Test"
                },
                "message_id": 1365,
                "text": "/start"
            },
            "update_id": 10000
        });
        serde_json::from_value::<CommonUpdate>(data).unwrap();
    }
}
