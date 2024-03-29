pub use http;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{
    params::ToParams,
    proto::{CommonUpdate, Message, WebhookInfo},
    request::{
        CopyMessageRequest, DeleteMessageRequest, DeleteWebhookRequest, ForwardMessageRequest,
        GetUpdatesRequest, SendAnimationRequest, SendChatActionRequest, SendMessageRequest,
        SendPhotoRequest, SetWebhookRequest,
    },
    response::MessageIdResponse,
};

pub trait Endpoint {
    type Request: ToParams;
    type Response;

    const METHOD: Method;
    const PATH: &'static str;
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Empty;

pub struct SendMessage;
impl Endpoint for SendMessage {
    type Request = SendMessageRequest;
    type Response = Message;

    const METHOD: Method = Method::GET;
    const PATH: &'static str = "sendMessage";
}

pub struct GetUpdates;
impl Endpoint for GetUpdates {
    type Request = GetUpdatesRequest;
    type Response = Vec<CommonUpdate>;

    const METHOD: Method = Method::GET;
    const PATH: &'static str = "getUpdates";
}

pub struct SetWebhook;
impl Endpoint for SetWebhook {
    type Request = SetWebhookRequest;
    type Response = bool;

    const METHOD: Method = Method::GET;
    const PATH: &'static str = "setWebhook";
}

pub struct DeleteWebhook;
impl Endpoint for DeleteWebhook {
    type Request = DeleteWebhookRequest;
    type Response = bool;

    const METHOD: Method = Method::GET;
    const PATH: &'static str = "deleteWebhook";
}

pub struct SendPhoto;

impl Endpoint for SendPhoto {
    type Request = SendPhotoRequest;
    type Response = Message;

    const METHOD: Method = Method::POST;
    const PATH: &'static str = "sendPhoto";
}

pub struct ForwardMessage;

impl Endpoint for ForwardMessage {
    type Request = ForwardMessageRequest;
    type Response = Message;

    const METHOD: Method = Method::POST;
    const PATH: &'static str = "forwardMessage";
}

pub struct CopyMessage;

impl Endpoint for CopyMessage {
    type Request = CopyMessageRequest;
    type Response = MessageIdResponse;

    const METHOD: Method = Method::POST;
    const PATH: &'static str = "copyMessage";
}

pub struct SendChatAction;

impl Endpoint for SendChatAction {
    type Request = SendChatActionRequest;
    type Response = bool;

    const METHOD: Method = Method::POST;
    const PATH: &'static str = "sendChatAction";
}

pub struct DeleteMessage;

impl Endpoint for DeleteMessage {
    type Request = DeleteMessageRequest;
    type Response = bool;

    const METHOD: Method = Method::POST;
    const PATH: &'static str = "deleteMessage";
}

pub struct SendAnimation;

impl Endpoint for SendAnimation {
    type Request = SendAnimationRequest;
    type Response = Message;

    const METHOD: Method = Method::POST;
    const PATH: &'static str = "sendAnimation";
}

pub struct GetWebhookInfo;

impl Endpoint for GetWebhookInfo {
    type Request = Empty;
    type Response = WebhookInfo;
    const METHOD: Method = Method::POST;
    const PATH: &'static str = "getWebhookInfo";
}
