use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub source: String,
    pub quote: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DraftPoint {
    pub text: String,
    pub evidence: Vec<Anchor>,
    pub required: Vec<Anchor>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourcePoint {
    pub id: String,
    pub point: DraftPoint,
}
