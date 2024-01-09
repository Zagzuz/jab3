use crate::{request::ImageGenerationRequest, response::ImageGenerationResponse};
use api::endpoints::{http::Method, Endpoint};

pub struct ImageGeneration;

impl Endpoint for ImageGeneration {
    type Request = ImageGenerationRequest;
    type Response = ImageGenerationResponse;
    const METHOD: Method = Method::POST;
    const PATH: &'static str = "v2/image/generation";
}
