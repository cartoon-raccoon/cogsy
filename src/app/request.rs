use reqwest;
use serde_json::Value;
use std::fs::read_to_string;
use std::error::Error;

use crate::app::response;

struct Release {

}



/*
* This module handles Discogs API requests and JSON deserialization
* The one main function exposed by this module is query();
* It returns a native Rust data structure that can be parsed without
* any form of (de)serialization or conversion
*/

pub fn query() -> String {
    let result = deserialize("discogs_collection.json").unwrap();
    result
}

fn request() {

}

fn deserialize(filepath: &str) -> Result<String, Box<dyn Error>> {
    let contents = read_to_string(filepath)?;
    let response: Value = serde_json::from_str(&contents)?;
    let releaselist = &response["releases"];
    Ok(contents)
}