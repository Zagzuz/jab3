use crate::{request::ChatCompletionsRequest, response::ChatCompletionsResponse};
use api::endpoints::{http, Endpoint};

pub struct ChatCompletions;

impl Endpoint for ChatCompletions {
    type Request = ChatCompletionsRequest;
    type Response = ChatCompletionsResponse;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "chat/completions";
}
