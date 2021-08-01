use std::{
    fs::read_to_string,
    fmt,
    rc::Rc,
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

use crate::app::{
    Release,
    message::Message
};
use crate::utils;
use crate::CONFIG;

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
    IOError,
    ServerError,
    NotFoundError,
    AuthorizationError,
    UnknownError,
    JSONParseError,
    CSVParseError(String),
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
            UpdateError::IOError => {
                write!(f, "An error occurred while reading from a file.")
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
            UpdateError::JSONParseError => {
                write!(f, "Error: Could not parse data from Discogs. Please try updating again.")
            }
            UpdateError::CSVParseError(s) => {
                write!(f, "Error: Could not parse CSV data: {}", s)
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
        UpdateError::JSONParseError
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(_error: std::io::Error) -> Self {
        UpdateError::IOError
    }
}

pub fn query_discogs(requester: &Client, url: &str) -> Result<String, UpdateError> {
    match requester.get(url).send() {
        Ok(response) => {
            match response.status() {
                StatusCode::NOT_FOUND => Err(UpdateError::NotFoundError),
                StatusCode::UNAUTHORIZED => Err(UpdateError::AuthorizationError),
                StatusCode::INTERNAL_SERVER_ERROR => Err(UpdateError::ServerError),
                StatusCode::OK => Ok(response.text().unwrap()),
                _ => {Err(UpdateError::UnknownError)}
            }
        }
        Err(_) => {
            Err(UpdateError::NetworkError)
        }
    }
}

pub fn build_client(token: &str) -> Client {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(token).unwrap());
    Client::builder()
        .user_agent("cogsy")
        .default_headers(headers)
        .build()
        .unwrap()
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

pub fn parse_releases(
    parse: Rc<ParseType>, 
    text: &str, 
    from_file: bool, c: bool,
    name: &str
) -> Result<Vec<Release>, UpdateError> {
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

    let to_index = match parse.as_ref() {
        ParseType::Collection => String::from("releases"),
        ParseType::Wantlist => String::from("wants"),
        _ => String::from("releases")
    };

    let releases_raw = response.get(&to_index).ok_or(UpdateError::JSONParseError)?;
    if let Value::Array(result) = releases_raw {
        let releaselist = result;

        //deserialization happens here
        for entry in releaselist {
            let id_no = entry.get("id")
                .ok_or(UpdateError::JSONParseError)?
                .as_u64()
                .ok_or(UpdateError::JSONParseError)?;
            let date_raw = entry.get("date_added")
                .ok_or(UpdateError::JSONParseError)?
                .as_str().ok_or(UpdateError::JSONParseError)?;
            //TODO: impl from for ParseResult
            let added_date = DateTime::parse_from_rfc3339(date_raw)
                .unwrap_or_else(|_| utils::get_utc_now()
                .with_timezone(&CONFIG.timezone()));
            let info = entry.get("basic_information").unwrap();

            
            //TODO: Figure out how to do this functionally
            let mut label_names = Vec::<String>::new();
            let labels = info["labels"].as_array()
                .ok_or(UpdateError::JSONParseError)?;
            for label in labels {
                label_names.push(label["name"].as_str()
                    .ok_or(UpdateError::JSONParseError)?
                    .to_string());
            }

            let mut formats = Vec::<String>::new();
            let formatlist = info["formats"].as_array()
                .ok_or(UpdateError::JSONParseError)?;
            for format in formatlist {
                let mut name = format["name"].as_str()
                    .ok_or(UpdateError::JSONParseError)?.to_string();
                let mut qty = format["qty"].as_str()
                    .ok_or(UpdateError::JSONParseError)?.to_string();
                let text = format["text"].as_str()
                    .unwrap_or("").to_string();
                if name == "Vinyl" {
                    qty.push_str("LP");
                }
                name.push(' ');
                name.push_str(&qty);
                if !text.is_empty() {
                    name.push_str(&format!(" ({})", text));
                }
                formats.push(name);
            }
            let title = info["title"].as_str()
                .ok_or(UpdateError::JSONParseError)?.to_string();
            let artist = info["artists"][0]["name"].as_str()
                .ok_or(UpdateError::JSONParseError)?
                .to_string();
            let search_string = unidecode(&title)
            .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

            if c {
                println!("'[{}] {}' by {}", Message::info(&name), title, artist)
            }

            releases.push(Release {
                id: id_no as i64,
                search_string,
                title,
                artist,
                year: info["year"].as_u64()
                    .ok_or(UpdateError::JSONParseError)? as u32,
                labels: label_names,
                formats,
                date_added: DateTime::<Utc>::from_utc(
                    added_date.naive_utc(), Utc
                )
            });
        }
    } else {return Err(UpdateError::JSONParseError);}
    Ok(releases)
}
