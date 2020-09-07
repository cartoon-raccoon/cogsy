use std::{
    collections::HashMap,
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
use crate::app::{Release, Folders};

/*
* This module handles Discogs API requests and JSON deserialization
* The one main function exposed by this module is fullupdate();
* It returns a native Rust data structure that can be parsed without
* any form of (de)serialization or conversion
*/

#[derive(Debug, Clone)]
pub enum ParseType {
    Initial,
    Folders(String),
    Collection,
    Wantlist(String),
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

//TODO: Handle the fucking unwraps
pub fn fullupdate(username: String, token: String) -> Result<Folders, QueryError> {
    let requester = build_client(token);
    let initial_url = build_url(ParseType::Initial, username.clone());
    let folders_raw: String;
    match query(&requester, &initial_url) {
        Ok(response) => {
            folders_raw = response;
        }
        Err(e) => {return Err(e)}
    }
    let mut folders = HashMap::<String, String>::new();

    let to_deserialize: Value = serde_json::from_str(&folders_raw).unwrap();
    let folders_raw = to_deserialize.get("folders").unwrap();
    if let Value::Array(_) = folders_raw {
        let folderlist = folders_raw.as_array().unwrap();
        for raw in folderlist.iter() {
            let foldername = raw.get("name")
                .unwrap().as_str()
                .unwrap().to_string();
            let folderurl = raw.get("resource_url")
                .unwrap().as_str()
                .unwrap().to_string();
            folders.insert(foldername, folderurl);
        } 
    }

    let mut master_folders: Folders = Folders::new();

    for (name, folderurl) in folders.iter() {
        let mut collection_url = build_url(ParseType::Folders(folderurl.clone()), username.clone());
        let mut master_vec: Vec<Release> = Vec::new();

        //* main update loop
        //TODO: Make this nicer and handle the unwraps
        loop { //I hate the stupid pyramid of doom here
            match query(&requester, &collection_url) {
                Ok(text) => {
                    let total: u64;
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
                                collection_url = pagination["urls"]["next"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string();
                            }
                        }
                        Err(_) => {return Err(QueryError::ParseError)}
                    }
                }
                Err(queryerror) => {return Err(queryerror)}
            }
        }
        master_folders.push(name.clone(), master_vec);
    }
    Ok(master_folders)
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

fn build_client(token: String) -> Client {
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
fn build_url(parse: ParseType, username: String) -> String {
    match parse {
        ParseType::Initial => {
            format!("https://api.discogs.com/users/{}/collection/folders", username)
        }
        //* Only use after Initial is called
        ParseType::Folders(url) => {
            format!("{}/releases?per_page=100", url)
        }
        ParseType::Collection => {
            format!("https://api.discogs.com/users/{}/collection/folders/0/releases?per_page=100", username)
        }
        ParseType::Wantlist(uid) => {
            format!("https://api.discogs.com/users/{}/wants", uid)
        }
    }
}

//#[allow(unused_assignments)]
//make this private once the database API is complete
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

    let to_index: String;
    match parse {
        ParseType::Collection => {
            to_index = String::from("releases");
        }
        ParseType::Wantlist(_) => {
            to_index = String::from("wants");
        }
        _ => {
            to_index = String::from("releases");
        }
    }

    //TODO: Change all the unwraps to handle errors you lazy fuck
    let releases_raw = response.get(&to_index).unwrap();
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
