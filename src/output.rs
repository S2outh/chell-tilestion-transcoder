use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// Metadata, additional input
#[derive(Deserialize)]
pub struct StreamMetadata {
    pub format: String,
    pub capacity: u64,
}

#[derive(Deserialize, Clone)]
pub struct TypeMetadata {
    pub telemetry_type: String,
    pub vector_keys: Option<Vec<String>>,
    pub flag_names: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Metadata {
    pub source: String,
    pub provider: Provider,
    pub stream_metadata: StreamMetadata,
    pub types: HashMap<String, TypeMetadata>,
    pub parsers: HashMap<String, String>,
}

// Tilestion format types
#[derive(Deserialize, Serialize)]
pub struct Connection {
    pub url: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub label: String,
    #[serde(alias = "source_kind")]
    pub source_kind: String,
    pub connection: Connection,
}

#[derive(Serialize)]
pub struct Subscription {
    pub subject: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub address: String,
    pub label: String,
    pub provider_id: String,
    pub subscription: Subscription,
    pub telemetry_type: String,
    pub format: String,
    pub parser_id: String,
    pub capacity: u64,
    pub publishable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_names: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Base {
    pub format_version: u32,
    pub version: String,
    pub generated_at: String,
    pub source: String,
    pub providers: Vec<Provider>,
    pub streams: Vec<Stream>,
}
