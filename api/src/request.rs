use crate::basic_types::{MessageId, MessageThreadId};
use compact_str::CompactString;
use derivative::Derivative;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::proto::{ChatAction, ChatId, MessageEntity, ParseMode, ReplyMarkup, UpdateType};

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub chat_id: ChatId,
    pub message_thread_id: Option<i64>,
    pub text: CompactString,
    pub parse_mode: Option<ParseMode>,
    pub entities: Option<Vec<MessageEntity>>,
    pub disable_web_page_preview: Option<bool>,
    pub disable_notification: Option<bool>,
    pub protect_content: Option<bool>,
    pub reply_to_message_id: Option<i32>,
    pub allow_sending_without_reply: Option<bool>,
    pub reply_markup: Option<ReplyMarkup>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct GetUpdatesRequest {
    pub offset: Option<usize>,
    pub limit: Option<u32>,
    pub timeout: Option<u32>,
    pub allowed_updates: Option<Vec<UpdateType>>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct DeleteWebhookRequest {
    drop_pending_updates: Option<bool>,
}

/// Use this method to send photos. On success, the sent Message is returned.
/// https://core.telegram.org/bots/api#sendphoto
#[skip_serializing_none]
#[derive(Debug, Derivative, Serialize)]
#[derivative(Default)]
pub struct SendPhotoRequest {
    pub chat_id: ChatId,
    pub message_thread_id: Option<i64>,
    /// Photo to send. Pass a file_id as String to send a photo that exists
    /// on the Telegram servers (recommended), pass an HTTP URL as a String for Telegram
    /// to get a photo from the Internet, or upload a new photo using multipart/form-data.
    /// The photo must be at most 10 MB in size. The photo's width and height must not exceed 10000 in total.
    /// Width and height ratio must be at most 20. [More information on Sending Files »](https://core.telegram.org/bots/api#sending-files)
    pub photo: Option<CompactString>,
    pub caption: Option<CompactString>,
    pub parse_mode: Option<ParseMode>,
    pub caption_entities: Option<Vec<MessageEntity>>,
    pub has_spoiler: Option<bool>,
    pub disable_notification: Option<bool>,
    pub protect_content: Option<bool>,
    pub reply_to_message_id: Option<i32>,
    pub allow_sending_without_reply: Option<bool>,
    pub reply_markup: Option<ReplyMarkup>,
}

/// Use this method to forward messages of any kind. Service messages can't be forwarded.
/// On success, the sent Message is returned.
/// https://core.telegram.org/bots/api#forwardmessage
#[skip_serializing_none]
#[derive(Debug, Derivative, Serialize)]
#[derivative(Default)]
pub struct ForwardMessageRequest {
    pub chat_id: ChatId,
    pub message_thread_id: Option<i64>,
    pub from_chat_id: ChatId,
    pub disable_notification: Option<bool>,
    pub protect_content: Option<bool>,
    pub message_id: MessageId,
}

/// Use this method to copy messages of any kind. Service messages and invoice messages can't be copied.
/// A quiz poll can be copied only if the value of the field correct_option_id is known to the bot.
/// The method is analogous to the method forwardMessage, but the copied message doesn't have a link
/// to the original message. Returns the MessageId of the sent message on success.
/// On success, the sent Message is returned.
/// https://core.telegram.org/bots/api#forwardmessage
#[skip_serializing_none]
#[derive(Debug, Derivative, Serialize)]
#[derivative(Default)]
pub struct CopyMessageRequest {
    pub chat_id: ChatId,
    pub message_thread_id: Option<MessageThreadId>,
    pub from_chat_id: ChatId,
    pub message_id: MessageId,
    pub caption: Option<CompactString>,
    pub parse_mode: Option<ParseMode>,
    pub caption_entities: Vec<MessageEntity>,
    pub disable_notification: Option<bool>,
    pub protect_content: Option<bool>,
    pub reply_to_message_id: Option<MessageId>,
    pub allow_sending_without_reply: Option<bool>,
    pub reply_markup: Option<ReplyMarkup>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct SendChatActionRequest {
    pub chat_id: ChatId,
    pub message_thread_id: Option<MessageThreadId>,
    pub action: ChatAction,
}

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct DeleteMessageRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

/// Use this method to send photos. On success, the sent Message is returned.
/// https://core.telegram.org/bots/api#sendanimation
#[skip_serializing_none]
#[derive(Debug, Derivative, Serialize)]
#[derivative(Default)]
pub struct SendAnimationRequest {
    pub chat_id: ChatId,
    pub message_thread_id: Option<i64>,
    /// Animation to send. Pass a file_id as String to send an animation that exists
    /// on the Telegram servers (recommended), pass an HTTP URL as a String
    /// for Telegram to get an animation from the Internet, or upload a new animation using multipart/form-data.
    /// [More information on Sending Files »](https://core.telegram.org/bots/api#sending-files)
    pub animation: Option<CompactString>,
    pub duration: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    /// Thumbnail of the file sent; can be ignored if thumbnail generation for the file is supported server-side.
    /// The thumbnail should be in JPEG format and less than 200 kB in size.
    /// A thumbnail's width and height should not exceed 320. Ignored if the file is not uploaded
    /// using multipart/form-data. Thumbnails can't be reused and can be only uploaded as a new file,
    /// so you can pass “attach://<file_attach_name>” if the thumbnail was uploaded
    /// using multipart/form-data under <file_attach_name>.
    /// [More information on Sending Files »](https://core.telegram.org/bots/api#sending-files)
    pub thumbnail: Option<CompactString>,
    pub caption: Option<CompactString>,
    pub parse_mode: Option<ParseMode>,
    pub caption_entities: Option<Vec<MessageEntity>>,
    pub has_spoiler: Option<bool>,
    pub disable_notification: Option<bool>,
    pub protect_content: Option<bool>,
    pub reply_to_message_id: Option<i32>,
    pub allow_sending_without_reply: Option<bool>,
    pub reply_markup: Option<ReplyMarkup>,
}
