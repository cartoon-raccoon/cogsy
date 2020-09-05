use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{self, HeaderMap};
use serde_json::Value;
use std::fs::read_to_string;
use std::error::Error;

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
    pub year: u64,
    pub labels: Vec<String>,
    pub formats: Vec<String>,
    pub date_added: String,
}

#[derive(Debug, Copy, Clone)]
pub enum ParseType {
    Collection,
    Wantlist,
}

#[allow(unused_assignments)]
pub fn query(parse: ParseType, filename: &str) -> Vec<Release> {
    let url = build_url(parse, String::from("cartoon.raccoon"));
    let requester = build_client();
    let response = requester.get(&url).send();
    let mut result = Vec::<Release>::new();
    match parse {
        ParseType::Collection => {
            result = parse_collection(filename).unwrap();
        }
        ParseType::Wantlist => {
            result = parse_wantlist(filename).unwrap();
        }
    }
    result
}

fn build_client() -> Client {
    let token = read_to_string("discogs_token").unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&token).unwrap());
    let requester = Client::builder()
                        .user_agent("cogsy")
                        .default_headers(headers)
                        .build()
                        .unwrap();
    requester
}

//builds a url based on its parsetype and user id
fn build_url(parse: ParseType, uid: String) -> String {
    match parse {
        ParseType::Collection => {
            format!("https://api.discogs.com/users/{}/collection/folder/0/releases", uid)
        }
        ParseType::Wantlist => {
            format!("https://api.discogs.com/users/{}/wants", uid)
        }
    }
}

#[allow(unused_assignments)]
fn parse_collection(filepath: &str) -> Result<Vec<Release>, Box<dyn Error>> {
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
    if let Value::Object(_) = pagination {
        total = pagination.get("items").unwrap().as_u64().unwrap();
    } else { //change this to handle the error instead of panicking
        panic!("Could not read json file properly.");
    }
    let releases_raw = response.get("releases").unwrap();
    if let Value::Array(_) = releases_raw {
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

            let mut formats = Vec::<String>::new();
            let formatlist = info["formats"].as_array().unwrap();
            for format in formatlist {
                let mut name = format["name"].as_str().unwrap().to_string();
                let mut qty = format["qty"].as_str().unwrap().to_string();
                if name == "Vinyl" {
                    qty.push_str("LP");
                }
                name.push_str(" ");
                name.push_str(&qty);
                formats.push(name);
            }

            releases.push(Release {
                id: id_no,
                title: info["title"].as_str().unwrap().to_string(),
                artist: info["artists"][0]["name"].as_str()
                    .unwrap()
                    .to_string(),
                year: info["year"].as_u64().unwrap(),
                labels: label_names,
                formats: formats,
                date_added: added_date
            });
        }
    } else {
        panic!("Release list could not be read");
    }
    Ok(releases)
}

fn parse_wantlist(filepath: &str) -> Result<Vec<Release>, Box<dyn Error>> {
    let releases = Vec::<Release>::new();
    Ok(releases)
}