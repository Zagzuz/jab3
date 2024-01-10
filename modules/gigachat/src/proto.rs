use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode, Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GigaChatRole {
    Assistant,
    User,
    System,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GigaChatMessage {
    pub role: GigaChatRole,
    pub content: CompactString,
}

impl Encode for GigaChatMessage {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.role, encoder)?;
        Encode::encode(&self.content.as_str(), encoder)?;
        Ok(())
    }
}

impl Decode for GigaChatMessage {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        #[derive(Decode)]
        struct Helper {
            pub role: GigaChatRole,
            pub content: String,
        }
        let helper = Helper {
            role: Decode::decode(decoder)?,
            content: Decode::decode(decoder)?,
        };
        Ok(Self {
            role: helper.role,
            content: helper.content.into(),
        })
    }
}
