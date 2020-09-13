use std::{
    fs::read_to_string,
    error::Error,
    fmt
};
use reqwest::{
    blocking::Client,
    header::{self, HeaderMap},
    StatusCode,
};
use serde_json::Value;
use chrono::DateTime;
use crate::app::Release;

/*
* This module handles Discogs API requests and JSON deserialization
* The one main function exposed by this module is fullupdate();
* It returns a native Rust data structure that can be parsed without
* any form of (de)serialization or conversion
*/

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ParseType {
    Initial,
    Profile,
    Folders(String),
    Collection,
    Wantlist,
}

#[derive(Debug, Clone)]
pub enum QueryError {
    NetworkError,
    ServerError,
    NotFoundError,
    AuthorizationError,
    UnknownError,
    ParseError,
    DBWriteError(String),
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
                write!(f, "Error: Discogs returned a 404. Check your username.")
            }
            QueryError::AuthorizationError => {
                write!(f, "Error: Discogs could not authorize your request. Check your token.")
            }
            QueryError::UnknownError => {
                write!(f, "An unknown error occurred. Check the logs for more info.")
            }
            QueryError::ParseError => {
                write!(f, "Error: Could not parse data from Discogs. Please try updating again.")
            }
            QueryError::DBWriteError(e) => {
                write!(f, "Database error: {}", e.to_string())
            }
        }
    }
}

pub fn query_discogs(requester: &Client, url: &String) -> Result<String, QueryError> {
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

pub fn build_client(token: String) -> Client {
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
pub fn build_url(parse: ParseType, username: String) -> String {
    match parse {
        ParseType::Initial => {
            format!("https://api.discogs.com/users/{}/collection/folders", username)
        }
        ParseType::Profile => {
            format!("https://api.discogs.com/users/{}", username)
        }
        //* Only use after Initial is called
        ParseType::Folders(url) => {
            format!("{}/releases?per_page=100", url)
        }
        ParseType::Collection => {
            format!("https://api.discogs.com/users/{}/collection/folders/0/releases?per_page=100", username)
        }
        ParseType::Wantlist => {
            format!("https://api.discogs.com/users/{}/wants", username)
        }
    }
}

pub fn parse_releases(parse: ParseType, text: &str, from_file: bool) -> Result<Vec<Release>, Box<dyn Error>> {
    /*
    *Step 1: Obtain the total item count
    *Step 2: Index into "releases" and ensure it is an array
    *Step 3: Iterate over the array and read each entry into a vec
    *Step 4: Return the vec
    */

    let contents: String;
    let mut releases = Vec::new();

    //reading the json file
    if from_file {
        contents = read_to_string(text)?;
    } else {
        contents = text.to_string();
    }
    let response: Value = serde_json::from_str(&contents)?;

    let to_index = match parse {
        ParseType::Collection => String::from("releases"),
        ParseType::Wantlist => String::from("wants"),
        _ => String::from("releases")
    };

    //TODO: Change all the unwraps to handle errors you lazy fuck
    let releases_raw = response.get(&to_index).unwrap();
    if let Value::Array(result) = releases_raw {
        let releaselist = result;

        //deserialization happens here
        for entry in releaselist {
            let id_no = entry.get("id").unwrap().as_u64().unwrap();
            let date_raw = entry.get("date_added").unwrap()
                .as_str().unwrap();
            let added_date = DateTime::parse_from_rfc3339(date_raw)
                .unwrap(); //TODO: Fix this unwrap
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
                let text = format["text"].as_str().unwrap_or("").to_string();
                if name == "Vinyl" {
                    qty.push_str("LP");
                }
                name.push_str(" ");
                name.push_str(&qty);
                if text.len() > 0 {
                    name.push_str(&format!(" ({})", text));
                }
                formats.push(name);
            }

            releases.push(Release {
                id: id_no as i64,
                title: info["title"].as_str().unwrap().to_string(),
                artist: info["artists"][0]["name"].as_str()
                    .unwrap()
                    .to_string(),
                year: info["year"].as_u64().unwrap() as u32,
                labels: label_names,
                formats: formats,
                date_added: added_date
            });
        }
    } else {return Err(Box::new(QueryError::ParseError));}
    Ok(releases)
}
