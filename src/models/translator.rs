use serde::{Deserialize, Serialize};

pub type WordData = Vec<Variant>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub featured: bool,
    pub text: String,
    pub pos: String,
    #[serde(rename = "audio_links")]
    pub audio_links: Vec<AudioLink>,
    pub translations: Vec<Translation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioLink {
    pub url: String,
    pub lang: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translation {
    pub featured: bool,
    pub text: String,
    pub pos: String,
    #[serde(rename = "audio_links")]
    pub audio_links: Option<Vec<AudioLink>>,
    pub examples: Vec<Example>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Example {
    pub src: String,
    pub dst: String,
}
