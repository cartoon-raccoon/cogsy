use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap};
use reqwest::StatusCode;
use serde_json::Value;

use std::fs::read_to_string;
use std::error::Error;
use std::fmt;

use crate::app::Release;

/*
* This module handles Discogs API requests and JSON deserialization
* The one main function exposed by this module is query();
* It returns a native Rust data structure that can be parsed without
* any form of (de)serialization or conversion
*/

#[derive(Debug, Copy, Clone)]
pub enum ParseType {
    Collection,
    Wantlist,
}

#[derive(Debug, Copy, Clone)]
pub enum QueryError {
    NetworkError,
    ServerError,
    NotFoundError,
    AuthorizationError,
    UnknownError,
    ParseError,
    DBWriteError,
}

impl std::error::Error for QueryError {}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueryError::NetworkError => {
                write!(f, "A network error occurred. Check your internet and try again.")
            }
            QueryError::ServerError => {
                write!(f, "The Discogs server encountered an error. Try again later.")
            }
            QueryError::NotFoundError => {
                write!(f, "Discogs returned a 404 error. Check your username.")
            }
            QueryError::AuthorizationError => {
                write!(f, "Discogs could not authorize your request. Check your token.")
            }
            QueryError::UnknownError => {
                write!(f, "An unknown error occurred. Check the logs for more info.")
            }
            QueryError::ParseError => {
                write!(f, "Could not parse data from Discogs. Please try updating again.")
            }
            QueryError::DBWriteError => {
                write!(f, "There was an error writing to the database. Try updating again.")
            }
        }
    }
}

#[allow(unused_assignments)]
//I hate the stupid pyramid of doom here
/*
*Steps:
*1. Enumerate the folders
*   - Make a request to the folders endpoint and parse it into a vec of folder names

*--------COMPLETE--------
*2. Iterate over the folders vector and make requests to each folder's contents
*   - Query the folder's metadata and get its count
*   - Set a tracking variable and initialize to 0
*   - Loop until the folder's contents are fully captured: while tracker < total
*--------COMPLETE--------
*
*3. Insert into Folders struct
*4. Repeat until all folders have been requested
*5. Return the Folders struct/read into database
*/
pub fn fullupdate(parse: ParseType) -> Result<Vec<Release>, QueryError> {
    let mut url = build_url(parse, String::from("cartoon.raccoon"));
    let requester = build_client();
    let mut master_vec: Vec<Release> = Vec::new();

    //* main update loop
    //TODO: Make this nicer and handle the unwraps
    loop {
        match query(&requester, &url) {
            Ok(text) => {
                let mut total: u64 = 0;
                let response: Value = serde_json::from_str(&text).unwrap();
                let pagination = response.get("pagination").unwrap();
                if let Value::Object(_) = pagination {
                    total = pagination.get("items").unwrap().as_u64().unwrap();
                } else { //change this to handle the error instead of panicking
                    panic!("Could not read json file properly.");
                }
                match parse_releases(ParseType::Collection, &text, false) {
                    Ok(mut releases) => {
                        master_vec.append(&mut releases);
                        if master_vec.len() as u64 == total {
                            break;
                        } else {
                            url = pagination["urls"]["next"].as_str().unwrap().to_string();
                        }
                    }
                    Err(_) => {return Err(QueryError::ParseError)}
                }
            }
            Err(queryerror) => {return Err(queryerror)}
        }
    }
    Ok(master_vec)
}

fn query(requester: &Client, url: &String) -> Result<String, QueryError> {
    match requester.get(url).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::NOT_FOUND => {
                    return Err(QueryError::NotFoundError)}
                StatusCode::UNAUTHORIZED => {
                    return Err(QueryError::AuthorizationError)}
                StatusCode::INTERNAL_SERVER_ERROR => {
                    return Err(QueryError::ServerError)}
                StatusCode::OK => {
                    return Ok(response.text().unwrap())
                }
                _ => {return Err(QueryError::UnknownError)}
            };
        }
        Err(_) => {
            return Err(QueryError::NetworkError)
        }
    }
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
            format!("https://api.discogs.com/users/{}/collection/folders/0/releases?per_page=100", uid)
        }
        ParseType::Wantlist => {
            format!("https://api.discogs.com/users/{}/wants", uid)
        }
    }
}

#[allow(unused_assignments)]
//make this private once the database API is complete
pub fn parse_releases(parse: ParseType, text: &str, from_file: bool) -> Result<Vec<Release>, Box<dyn Error>> {
    /*
    *Step 1: Obtain the total item count
    *Step 2: Index into "releases" and ensure it is an array
    *Step 3: Iterate over the array and read each entry into a vec
    *Step 4: Return the vec
    */

    //defining tracking variables
    let mut total = 0;
    let mut contents = String::new();
    let mut releases = Vec::new();

    //reading the json file
    if from_file {
        contents = read_to_string(text)?;
    } else {
        contents = text.to_string();
    }
    let response: Value = serde_json::from_str(&contents)?;

    //TODO: Change all the unwraps to handle errors you lazy fuck
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