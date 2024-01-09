use crate::{endpoints::ImageGeneration, request::ImageGenerationProvider};
use compact_str::CompactString;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Success,
    Fail,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ImageGenerationResult {
    Fail(ImageGenerationFail),
    Success(ImageGenerationInfo),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EdenResponse {
    ImageGenerationResponse(ImageGenerationResponse),
    Error(ResponseError),
}

#[derive(Debug, Deserialize)]
pub struct ResponseError {
    pub error: ResponseErrorData,
}

#[derive(Debug, Deserialize)]
pub struct ResponseErrorData {
    pub r#type: CompactString,
    pub message: ResponseErrorMessage,
}

#[derive(Debug, Deserialize)]
pub struct ResponseErrorMessage {
    pub fallback_providers: Vec<CompactString>,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationResponse(pub HashMap<ImageGenerationProvider, ImageGenerationResult>);

#[derive(Debug, Deserialize)]
pub struct ImageGenerationFail {
    pub error: ImageGenerationErrorInfo,
    pub provider_status_code: i32,
    pub cost: f32,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationErrorInfo {
    pub message: CompactString,
    pub r#type: CompactString,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationInfo {
    pub items: Vec<ImageGenerationItem>,
    pub cost: f32,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationItem {
    pub image: CompactString,
    pub image_resource_url: CompactString,
}

#[cfg(test)]
mod tests {
    use crate::response::EdenResponse;

    #[test]
    fn deserialize_eden_response() {
        let v = serde_json::json!({
            "openai": {
                "error": {
                    "message": "Openai has returned an error: {\
                        \"error\": {\
                            \"code\": \"invalid_size\",\
                            \"message\": \"The size is not supported by this model.\",\
                            \"param\": null,\
                            \"type\": \"invalid_request_error\"\
                        }\
                    }",
                    "type": "ProviderException"
                },
                "status": "fail",
                "provider_status_code": 400,
                "cost": 0.0
            },
            "stabilityai": {
                "status": "success",
                "items":[{
                    "image":"iVBORw0KGgoAAAANSUhEUgAAAgAAAAIACAIAA",
                    "image_resource_url":"pNU6UbaLGhCwE9n5J&Key-Pair-Id=K1F55BTI9AHGIK"
                }],
                "is_fallback": true,
                "cost": 0.004
            }
        });
        let _response = serde_json::from_value::<EdenResponse>(v).unwrap();
    }
}
