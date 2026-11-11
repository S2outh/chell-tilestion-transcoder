use std::collections::HashMap;

use serde::{Deserialize, de::IgnoredAny};

#[derive(Deserialize)]
pub struct ParsedDef {
    pub label: String,
    pub address: String,
    #[serde(rename = "type")]
    pub ty: String,
}

// chell doc format types
#[derive(Deserialize)]
pub struct Def {
    pub label: String,
    pub address: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub command: bool,
    pub parsed: Option<Vec<ParsedDef>>,
}

// Rustdoc format types
#[derive(Deserialize)]
pub struct Module {
    pub items: Vec<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InnerElement {
    Module(Module),
    Struct(IgnoredAny),
    #[serde(untagged)]
    Other(IgnoredAny),
}

#[derive(Deserialize)]
pub struct Element {
    pub name: Option<String>,
    pub docs: Option<String>,
    pub inner: InnerElement,
}

#[derive(Deserialize)]
pub struct Base {
    pub root: u64,
    pub index: HashMap<u64, Element>,
}
