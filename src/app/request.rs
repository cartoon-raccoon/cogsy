use reqwest;
use serde_json::Value;
use std::fs::read_to_string;
use std::error::Error;

use crate::app::response::Response;

/*
* This module handles Discogs API requests and JSON deserialization
* The one main function exposed by this module is query();
* It returns a native Rust data structure that can be parsed without
* any form of (de)serialization or conversion
*/

#[derive(Debug, Clone)]
pub struct Release {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub labels: Vec<String>,
    pub date_added: String,
}

pub fn query(filename: &str) -> Vec<Release> {
    let result = deserialize(filename).unwrap();
    result
}

fn request() {
    //requests to the Discogs API made here
}

fn deserialize(filepath: &str) -> Result<Vec<Release>, Box<dyn Error>> {
    /*
    *Step 1: Obtain the total item count
    *Step 2: Index into "releases" and ensure it is an array
    *Step 3: Iterate over the array and read each entry into a vec
    *Step 4: Return the vec
    TODO: Implement recursive querying for results with multiple pages
    TODO: Add a closure to show an error message on the commandline
    */

    //defining tracking variables
    let mut total = 0;
    let mut releases = Vec::new();

    //reading the json file
    let contents = read_to_string(filepath)?;
    let response: Value = serde_json::from_str(&contents)?;

    let pagination = response.get("pagination").unwrap();
    if let Value::Object(Map) = pagination {
        total = pagination.get("items").unwrap().as_u64().unwrap();
    } else { //change this to handle the error instead of panicking
        panic!("Could not read json file properly.");
    }
    let releases_raw = response.get("releases").unwrap();
    if let Value::Array(Vec) = releases_raw {
        let releaselist = releases_raw.as_array().unwrap();

        //deserialization happens here
        for entry in releaselist {
            let id_no = entry.get("id").unwrap().as_u64().unwrap();
            let added_date = entry.get("date_added").unwrap()
                .as_str().unwrap()
                .to_string();
            let info = entry.get("basic_information").unwrap();
            
            //TODO: Figure out how to do this functionally
            let mut label_names = Vec::<String>::new();
            let labels = info["labels"].as_array().unwrap();
            for label in labels {
                label_names.push(label["name"].as_str()
                    .unwrap()
                    .to_string());
            }

            releases.push(Release {
                id: id_no,
                title: info["title"].as_str().unwrap().to_string(),
                artist: info["artists"][0]["name"].as_str()
                    .unwrap()
                    .to_string(),
                labels: label_names,
                date_added: added_date
            });
        }
    } else {
        panic!("Release list could not be read");
    }
    Ok(releases)
}