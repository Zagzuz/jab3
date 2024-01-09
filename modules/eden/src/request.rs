use compact_str::CompactString;
use derive_more::Display;
use serde::{ser::Error, Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ImageGenerationRequest {
    providers: ProvidersList,
    fallback_providers: Option<ProvidersList>,
    response_as_dict: Option<bool>,
    attributes_as_list: Option<bool>,
    show_original_response: Option<bool>,
    settings: ImageGenerationSettings,
    text: CompactString,
    resolution: Resolution,
    /// The number of images to generate. Must be between 1 and 10.
    num_images: u8,
}

impl ImageGenerationRequest {
    pub fn new(
        providers: ProvidersList,
        fallback_providers: Option<ProvidersList>,
        show_original_response: bool,
        settings: ImageGenerationSettings,
        text: CompactString,
        resolution: Resolution,
        num_images: u8,
    ) -> Self {
        Self {
            providers,
            fallback_providers,
            response_as_dict: Some(true),
            attributes_as_list: Some(false),
            show_original_response: Some(show_original_response),
            settings,
            text,
            resolution,
            num_images,
        }
    }
}

#[derive(Debug)]
pub struct ProvidersList(Vec<ImageGenerationProvider>);

impl From<Vec<ImageGenerationProvider>> for ProvidersList {
    fn from(value: Vec<ImageGenerationProvider>) -> Self {
        Self(value)
    }
}

impl Serialize for ProvidersList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0.is_empty() {
            serializer.serialize_str("")
        } else {
            let vs = self
                .0
                .iter()
                .map(|provider| format!("{provider}",))
                .collect::<Vec<_>>();
            serializer.serialize_str(&vs.join(","))
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ImageGenerationSettings(pub HashMap<ImageGenerationProvider, OpenAIModels>);

#[derive(Debug, Display, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ImageGenerationProvider {
    #[display(fmt = "deepai")]
    DeepAI,
    #[display(fmt = "openai")]
    OpenAI,
    #[display(fmt = "stabilityai")]
    StabilityAI,
    #[display(fmt = "replicate")]
    Replicate,
}

#[derive(Debug, Serialize)]
pub enum OpenAIModels {
    #[serde(rename = "dall-e-2")]
    Dalle2,
    #[serde(rename = "dall-e-3")]
    Dalle3,
}

#[derive(Debug, Serialize)]
pub enum StabilityAIModels {
    #[serde(rename = "esrgan-v1-x2plus")]
    EsrganV1X2Plus,
    #[serde(rename = "stable-diffusion-v1-6")]
    StableDiffusionV1_6,
    #[serde(rename = "stable-diffusion-xl-1024-v0-9")]
    StableDiffusionX1_1024v0_9,
    #[serde(rename = "stable-diffusion-xl-1024-v1-0")]
    StableDiffusionX1_1024v1_0,
    #[serde(rename = "stable-diffusion-xl-beta-v2-2-2")]
    StableDiffusionX1BetaV2_2_2,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplicateModels {
    AnimeStyle,
    Classic,
    VintedoisDiffusion,
}

#[derive(Debug, Serialize)]
pub enum Resolution {
    #[serde(rename = "256x256")]
    Res256_256,
    #[serde(rename = "512x512")]
    Res512_512,
    #[serde(rename = "1024x1024")]
    Res1024_1024,
}
