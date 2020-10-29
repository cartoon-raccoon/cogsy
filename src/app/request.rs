use std::{
    fs::read_to_string,
    fmt
};
use reqwest::{
    blocking::Client,
    header::{self, HeaderMap},
    StatusCode,
};
use serde_json::Value;
use chrono::{
    DateTime,
    Utc,
};
use unidecode::unidecode;

use crate::app::Release;
use crate::utils;

#[derive(Debug, Clone)]
pub enum ParseType {
    Initial,
    Profile,
    Folders(String),
    Collection,
    Wantlist,
}

#[derive(Debug, Clone)]
pub enum UpdateError {
    NetworkError,
    ServerError,
    NotFoundError,
    AuthorizationError,
    UnknownError,
    ParseError,
    ThreadPanicError,
    DBWriteError(String),
}

impl std::error::Error for UpdateError {}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::NetworkError => {
                write!(f, "A network error occurred. Check your internet and try again.")
            }
            UpdateError::ServerError => {
                write!(f, "The Discogs server encountered an error. Try again later.")
            }
            UpdateError::NotFoundError => {
                write!(f, "Error: Discogs returned a 404. Check your username.")
            }
            UpdateError::AuthorizationError => {
                write!(f, "Error: Discogs could not authorize your request. Check your token.")
            }
            UpdateError::UnknownError => {
                write!(f, "An unknown error occurred. Check the logs for more info.")
            }
            UpdateError::ParseError => {
                write!(f, "Error: Could not parse data from Discogs. Please try updating again.")
            }
            UpdateError::ThreadPanicError => {
                write!(f, "Error: Update thread panicked. Please try again.")
            }
            UpdateError::DBWriteError(e) => {
                write!(f, "Database error: {}", e.to_string())
            }
        }
    }
}

impl From<serde_json::Error> for UpdateError {
    fn from(_error: serde_json::Error) -> Self {
        UpdateError::ParseError
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(_error: std::io::Error) -> Self {
        UpdateError::ParseError
    }
}

pub fn query_discogs(requester: &Client, url: &String) -> Result<String, UpdateError> {
    match requester.get(url).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::NOT_FOUND => {
                    return Err(UpdateError::NotFoundError)}
                StatusCode::UNAUTHORIZED => {
                    return Err(UpdateError::AuthorizationError)}
                StatusCode::INTERNAL_SERVER_ERROR => {
                    return Err(UpdateError::ServerError)}
                StatusCode::OK => {
                    return Ok(response.text().unwrap())
                }
                _ => {return Err(UpdateError::UnknownError)}
            };
        }
        Err(_) => {
            return Err(UpdateError::NetworkError)
        }
    }
}

pub fn build_client(token: &str) -> Client {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(token).unwrap());
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

pub fn parse_releases(parse: ParseType, text: &str, from_file: bool) -> Result<Vec<Release>, UpdateError> {
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
    let releases_raw = response.get(&to_index).ok_or(UpdateError::ParseError)?;
    if let Value::Array(result) = releases_raw {
        let releaselist = result;

        //deserialization happens here
        for entry in releaselist {
            let id_no = entry.get("id")
                .ok_or(UpdateError::ParseError)?
                .as_u64()
                .ok_or(UpdateError::ParseError)?;
            let date_raw = entry.get("date_added")
                .ok_or(UpdateError::ParseError)?
                .as_str().ok_or(UpdateError::ParseError)?;
            //TODO: impl from for ParseResult
            let added_date = DateTime::parse_from_rfc3339(date_raw)
                .unwrap_or(utils::get_utc_now()
                .with_timezone(&utils::Config::timezone()));
            let info = entry.get("basic_information").unwrap();

            
            //TODO: Figure out how to do this functionally
            let mut label_names = Vec::<String>::new();
            let labels = info["labels"].as_array()
                .ok_or(UpdateError::ParseError)?;
            for label in labels {
                label_names.push(label["name"].as_str()
                    .ok_or(UpdateError::ParseError)?
                    .to_string());
            }

            let mut formats = Vec::<String>::new();
            let formatlist = info["formats"].as_array()
                .ok_or(UpdateError::ParseError)?;
            for format in formatlist {
                let mut name = format["name"].as_str()
                    .ok_or(UpdateError::ParseError)?.to_string();
                let mut qty = format["qty"].as_str()
                    .ok_or(UpdateError::ParseError)?.to_string();
                let text = format["text"].as_str()
                    .ok_or(UpdateError::ParseError)?.to_string();
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
            let title = info["title"].as_str()
                .ok_or(UpdateError::ParseError)?.to_string();
            let search_string = unidecode(&title)
            .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

            releases.push(Release {
                id: id_no as i64,
                search_string: search_string,
                title: title,
                artist: info["artists"][0]["name"].as_str()
                    .ok_or(UpdateError::ParseError)?
                    .to_string(),
                year: info["year"].as_u64()
                    .ok_or(UpdateError::ParseError)? as u32,
                labels: label_names,
                formats: formats,
                date_added: DateTime::<Utc>::from_utc(
                    added_date.naive_utc(), Utc
                )
            });
        }
    } else {return Err(UpdateError::ParseError);}
    Ok(releases)
}
