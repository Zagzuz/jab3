use crate::connector::{polling::PollingConnector, Connector};
use api::{
    basic_types::{MessageId, MessageThreadId},
    endpoints::{
        CopyMessage, DeleteMessage, ForwardMessage, SendAnimation, SendChatAction, SendMessage,
        SendPhoto,
    },
    proto::{ChatAction, ChatId, Message, MessageEntity, ParseMode, ReplyMarkup},
    request::{
        CopyMessageRequest, DeleteMessageRequest, ForwardMessageRequest, SendAnimationRequest,
        SendChatActionRequest, SendMessageRequest, SendPhotoRequest,
    },
    response::{CommonResponse, MessageIdResponse},
};
use async_trait::async_trait;
use compact_str::{CompactString, ToCompactString};
use eyre::eyre;
use std::sync::Arc;

#[async_trait]
pub trait Communicate: Send + Sync {
    async fn send_message(
        &self,
        text: &str,
        chat_id: ChatId,
    ) -> eyre::Result<CommonResponse<Message>>;

    async fn reply_message(
        &self,
        text: &str,
        chat_id: ChatId,
        reply_to_message_id: MessageId,
        parse_mode: Option<ParseMode>,
    ) -> eyre::Result<CommonResponse<Message>>;

    async fn send_photo_url(
        &self,
        url: &str,
        chat_id: ChatId,
        reply_to_message_id: Option<MessageId>,
    ) -> eyre::Result<CommonResponse<Message>>;

    async fn send_animation_url(
        &self,
        url: &str,
        chat_id: ChatId,
        reply_to_message_id: Option<MessageId>,
    ) -> eyre::Result<CommonResponse<Message>>;

    async fn forward_message(
        &self,
        to_chat_id: ChatId,
        from_chat_id: ChatId,
        message_id: MessageId,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
    ) -> eyre::Result<CommonResponse<Message>>;

    #[allow(clippy::too_many_arguments)]
    async fn copy_message(
        &self,
        chat_id: ChatId,
        message_thread_id: Option<MessageThreadId>,
        from_chat_id: ChatId,
        message_id: MessageId,
        caption: Option<CompactString>,
        parse_mode: Option<ParseMode>,
        caption_entities: Vec<MessageEntity>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        reply_to_message_id: Option<MessageId>,
        allow_sending_without_reply: Option<bool>,
        reply_markup: Option<ReplyMarkup>,
    ) -> eyre::Result<CommonResponse<MessageIdResponse>>;

    async fn send_chat_action(
        &self,
        chat_id: ChatId,
        message_thread_id: Option<MessageThreadId>,
        action: ChatAction,
    ) -> eyre::Result<CommonResponse<bool>>;

    async fn delete_message(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
    ) -> eyre::Result<CommonResponse<bool>>;
}

#[derive(Clone)]
pub struct Communicator {
    token: Arc<CompactString>,
}

impl Communicator {
    pub fn new(token: &str) -> Self {
        Self {
            token: Arc::new(token.into()),
        }
    }
}

impl Communicator {
    pub(crate) async fn del(&self, message: &Message) -> eyre::Result<bool> {
        let requested_message = message
            .reply_to_message
            .as_ref()
            .ok_or(eyre!("replied message does not exist to delete"))?;
        let requested_message_deleted = self
            .delete_message(
                requested_message.chat.id.into(),
                requested_message.message_id,
            )
            .await?
            .into_result()?;
        // let command_message_deleted = self
        //     .delete_message(message.chat.id.into(), message.message_id)
        //     .await?
        //     .into_result()?;
        Ok(
            requested_message_deleted, /* && command_message_deleted*/
        )
    }
}

#[async_trait]
impl Communicate for Communicator {
    async fn send_message(
        &self,
        text: &str,
        chat_id: ChatId,
    ) -> eyre::Result<CommonResponse<Message>> {
        let request = SendMessageRequest {
            text: text.to_compact_string(),
            parse_mode: None,
            entities: None,
            disable_web_page_preview: None,
            disable_notification: None,
            chat_id,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            message_thread_id: None,
            protect_content: None,
            reply_markup: None,
        };
        Ok(PollingConnector::send_request::<SendMessage>(&self.token, &request, None).await?)
    }

    async fn reply_message(
        &self,
        text: &str,
        chat_id: ChatId,
        reply_to_message_id: MessageId,
        parse_mode: Option<ParseMode>,
    ) -> eyre::Result<CommonResponse<Message>> {
        let request = SendMessageRequest {
            text: text.to_compact_string(),
            parse_mode,
            entities: None,
            disable_web_page_preview: None,
            disable_notification: None,
            chat_id,
            reply_to_message_id: Some(reply_to_message_id),
            allow_sending_without_reply: None,
            message_thread_id: None,
            protect_content: None,
            reply_markup: None,
        };
        PollingConnector::send_request::<SendMessage>(&self.token, &request, None).await
    }

    async fn send_photo_url(
        &self,
        url: &str,
        chat_id: ChatId,
        reply_to_message_id: Option<MessageId>,
    ) -> eyre::Result<CommonResponse<Message>> {
        let request = SendPhotoRequest {
            photo: Some(url.to_compact_string()),
            chat_id,
            reply_to_message_id,
            ..Default::default()
        };
        PollingConnector::send_request::<SendPhoto>(&self.token, &request, None).await
    }

    async fn send_animation_url(
        &self,
        url: &str,
        chat_id: ChatId,
        reply_to_message_id: Option<MessageId>,
    ) -> eyre::Result<CommonResponse<Message>> {
        let request = SendAnimationRequest {
            animation: Some(url.to_compact_string()),
            chat_id,
            reply_to_message_id,
            ..Default::default()
        };
        PollingConnector::send_request::<SendAnimation>(&self.token, &request, None).await
    }

    async fn forward_message(
        &self,
        chat_id: ChatId,
        from_chat_id: ChatId,
        message_id: MessageId,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
    ) -> eyre::Result<CommonResponse<Message>> {
        let request = ForwardMessageRequest {
            chat_id,
            message_thread_id: None,
            from_chat_id,
            disable_notification,
            protect_content,
            message_id,
        };
        PollingConnector::send_request::<ForwardMessage>(&self.token, &request, None).await
    }

    async fn copy_message(
        &self,
        chat_id: ChatId,
        message_thread_id: Option<MessageThreadId>,
        from_chat_id: ChatId,
        message_id: MessageId,
        caption: Option<CompactString>,
        parse_mode: Option<ParseMode>,
        caption_entities: Vec<MessageEntity>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        reply_to_message_id: Option<MessageId>,
        allow_sending_without_reply: Option<bool>,
        reply_markup: Option<ReplyMarkup>,
    ) -> eyre::Result<CommonResponse<MessageIdResponse>> {
        let request = CopyMessageRequest {
            chat_id,
            message_thread_id,
            from_chat_id,
            message_id,
            caption,
            parse_mode,
            caption_entities,
            disable_notification,
            protect_content,
            reply_to_message_id,
            allow_sending_without_reply,
            reply_markup,
        };
        PollingConnector::send_request::<CopyMessage>(&self.token, &request, None).await
    }

    async fn send_chat_action(
        &self,
        chat_id: ChatId,
        message_thread_id: Option<MessageThreadId>,
        action: ChatAction,
    ) -> eyre::Result<CommonResponse<bool>> {
        let request = SendChatActionRequest {
            chat_id,
            message_thread_id,
            action,
        };
        PollingConnector::send_request::<SendChatAction>(&self.token, &request, None).await
    }

    async fn delete_message(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
    ) -> eyre::Result<CommonResponse<bool>> {
        let request = DeleteMessageRequest {
            chat_id,
            message_id,
        };
        PollingConnector::send_request::<DeleteMessage>(&self.token, &request, None).await
    }
}
