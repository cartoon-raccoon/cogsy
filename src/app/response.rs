use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_json::Value;

use std::iter::Map;

#[derive(Deserialize)]
pub struct Metadata {
    pagination: Value,
}

#[derive(Deserialize)]
pub struct Release {

}

#[derive(Deserialize)]
pub struct Response {
    metadata: Metadata,
    releases: Vec<Release>,
}
