use crate::basic_types::MessageId;
use compact_str::CompactString;
use serde::{de, Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum CommonResponse<R> {
    Ok(R),
    Err(ErrorResponse),
}

#[allow(clippy::from_over_into)]
impl<R> Into<Result<R, ErrorResponse>> for CommonResponse<R> {
    fn into(self) -> Result<R, ErrorResponse> {
        match self {
            CommonResponse::Ok(response) => Ok(response),
            CommonResponse::Err(error) => Err(error),
        }
    }
}

impl<R> CommonResponse<R> {
    pub fn into_result(self) -> Result<R, ErrorResponse> {
        self.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub description: CompactString,
    pub error_code: i64,
    pub migrate_to_chat_id: Option<i64>,
    pub retry_after: Option<i64>,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "response error: {}, code: {}",
            self.description, self.error_code
        )
    }
}

impl Error for ErrorResponse {
    fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl<'de, R: Deserialize<'de>> Deserialize<'de> for CommonResponse<R> {
    fn deserialize<D>(deserializer: D) -> Result<CommonResponse<R>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = Map::deserialize(deserializer)?;

        let ok = map
            .remove("ok")
            .ok_or_else(|| de::Error::missing_field("ok"))
            .map(Deserialize::deserialize)?
            .map_err(de::Error::custom)?;
        if ok {
            let result = map
                .remove("result")
                .ok_or_else(|| de::Error::missing_field("result"))
                .map(R::deserialize)?
                .map_err(de::Error::custom)?;
            Ok(CommonResponse::Ok(result))
        } else {
            let rest = Value::Object(map);
            ErrorResponse::deserialize(rest)
                .map(CommonResponse::Err)
                .map_err(de::Error::custom)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MessageIdResponse {
    pub message_id: MessageId,
}

#[cfg(test)]
mod tests {
    use crate::{proto::Message, response::CommonResponse};
    use serde_json::json;

    #[test]
    fn deserialize_response_check() {
        let message = json!({
            "ok": true,
            "result": {
                "message_id": 123,
                "date": 2345,
                "chat": {
                    "id": 1,
                    "type": "group",
                }
            }
        });
        serde_json::from_value::<CommonResponse<Message>>(message).unwrap();

        let message = json!({"ok":true,"result":true,"description":"Webhook was set"});
        serde_json::from_value::<CommonResponse<bool>>(message).unwrap();
    }
}
