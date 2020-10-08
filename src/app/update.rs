use std::{
    path::Path,
    thread,
    collections::HashMap,
};

use serde_json::Value;
use reqwest::blocking::Client;
use chrono::{DateTime, Utc};

use crate::app::{
    {Release, Folders, Profile},
    request::*,
    database::{admin, update, purge},
    message::{Message, MessageKind},
};
use crate::utils;

pub fn full(username: &str, token: &str, from_cmd: bool, debug: bool) -> Result<(), QueryError> {
    if Path::new(&utils::database_file()).exists() {
        match admin::check_integrity() {
            true => {},
            false => {
                if debug {println!("Database integrity check failed, purging and refreshing now.")}
                match purge::complete() {
                    Ok(()) => {},
                    Err(e) => {return Err(QueryError::DBWriteError(e.to_string()))}
                }
            }
        }
    }
    match admin::init_db() {
        Ok(_) => {},
        Err(e) => {return Err(QueryError::DBWriteError(e.to_string()))}
    }
    
    //* pulling data from Discogs
    let requester = build_client(&token);
    
    if from_cmd {print!("Updating profile...")}
    let profile = match profile(&requester, username) {
        Ok(profile) => profile,
        Err(e) => {return Err(e);}
    };
    if from_cmd {
        print!("{}", Message::set("     Success!", MessageKind::Success));
        print!("\nUpdating wantlist...")
    }
    let wantlist = match wantlist(requester.clone(), username) {
        Ok(wantlist) => wantlist,
        Err(e) => {return Err(e);}
    };
    if from_cmd {
        print!("{}", Message::set("    Success!", MessageKind::Success));
        print!("\nUpdating collection...")
    }
    let collection = match collection(requester, username) {
        Ok(collection) => collection,
        Err(e) => {return Err(e);}
    };
    if from_cmd {print!("{}", Message::set("  Success!", MessageKind::Success));}
    
    //* committing data to db
    let mut dbhandle = match update::DBHandle::new() {
        Ok(handle) => handle,
        Err(e) => {return Err(QueryError::DBWriteError(e.to_string()));}
    };

    if from_cmd {println!("\nWriting to database...\n")}
    match dbhandle.update_profile(profile) {
        Ok(_) => {},
        Err(e) => return Err(QueryError::DBWriteError(e.to_string()))
    }
    match dbhandle.update_wantlist(wantlist) {
        Ok(_) => {},
        Err(e) => return Err(QueryError::DBWriteError(e.to_string()))
    }
    match dbhandle.update_collection(collection) {
        Ok(_) => {},
        Err(e) => return Err(QueryError::DBWriteError(e.to_string())),
    }

    //* final integrity check
    match admin::check_integrity() {
        true => {},
        false => {
            let errormsg = String::from("Integrity check failed");
            return Err(QueryError::DBWriteError(errormsg))
        }
    }
    if from_cmd {
        println!("{}", Message::set("Database update successful.", MessageKind::Success));
    }
    Ok(())
}

pub fn profile(requester: &Client, username: &str) -> Result<Profile, QueryError> {
    let profile_url = build_url(ParseType::Profile, username.to_string());
    let master_prof: Profile;

    //pulling profile and deserialization
    let response = query_discogs(&requester, &profile_url)?;
    let profile_raw: Value = serde_json::from_str(&response)
                .unwrap_or(Value::Null);
    if let Value::Null = profile_raw {
        return Err(QueryError::ParseError)
    }
    master_prof = Profile {
        username: profile_raw["username"]
            .as_str().unwrap_or("undefined").to_string(),
        real_name: profile_raw["name"]
            .as_str().unwrap_or("undefined").to_string(),
        registered: DateTime::<Utc>::from_utc(
            DateTime::parse_from_rfc3339(
            profile_raw["registered"]
            .as_str().unwrap()
            ).unwrap().naive_utc(), Utc
        ),
        listings: profile_raw["num_for_sale"]
            .as_u64().unwrap_or(0) as u32,
        collection: profile_raw["num_collection"]
            .as_u64().unwrap_or(0) as u32,
        wantlist: profile_raw["num_wantlist"]
            .as_u64().unwrap_or(0) as u32,
        rated: profile_raw["releases_rated"]
            .as_u64().unwrap_or(0) as u32,
        average_rating: profile_raw["rating_avg"]
            .as_f64().unwrap_or(0.0),
    };
    Ok(master_prof)
}

pub fn wantlist(requester: Client, username: &str) -> Result<Vec<Release>, QueryError> {
    let wantlist_url = build_url(ParseType::Wantlist, username.to_string());
    let master_wants = get_full(requester, ParseType::Wantlist, wantlist_url)?;

    Ok(master_wants)
}

pub fn collection(requester: Client, username: &str) -> Result<Folders, QueryError> {
    //* 1a: Enumerating folders
    let initial_url = build_url(ParseType::Initial, username.to_string());
    let folders_raw = query_discogs(&requester, &initial_url)?;
    let mut folders = HashMap::<String, String>::new();

    let to_deserialize: Value = serde_json::from_str(&folders_raw)
        .unwrap_or(Value::Null);
    if let Value::Null = to_deserialize {
        return Err(QueryError::ParseError);
    }
    let folders_raw = to_deserialize.get("folders");
    if let Some(Value::Array(result)) = folders_raw {
        let folderlist = result;

        //not sure how to handle the unwraps here
        //generally shouldn't fail but idk for sure
        for raw in folderlist.iter() {
            let foldername = raw.get("name")
                .unwrap().as_str()
                .unwrap().to_string();
            let folderurl = raw.get("resource_url")
                .unwrap().as_str()
                .unwrap().to_string();
            folders.insert(foldername, folderurl);
        } 
    } else {return Err(QueryError::ParseError);}

    let mut master_folders: Folders = Folders::new();
    let mut threads = Vec::new();
    // let (tx, rx) = mpsc::channel();

    //*1b: Pulling each folder
    for (name, folderurl) in folders {
        let owned_uname = username.to_string();
        let req_clone = requester.clone();
        threads.push(thread::spawn( move || -> Result<(String, Vec<Release>), QueryError> {
            let collection_url = build_url(ParseType::Folders(folderurl), owned_uname);
            let releases = get_full(req_clone, ParseType::Collection, collection_url)?;
            Ok((name, releases))
        }));
    }
    for thread in threads {
        let (name, releases) = match thread.join() {
            Ok(result) => result?,
            Err(_) => {return Err(QueryError::ThreadPanicError);}
        };
        master_folders.push(name, releases);
    };
    Ok(master_folders)
}

fn get_full(client: Client, parse: ParseType, starting_url: String) -> Result<Vec<Release>, QueryError> {
    let mut url = starting_url;
    let mut master_vec: Vec<Release> = Vec::new();

    //main update loop
    //TODO: Make this nicer and handle the unwraps
    loop { //I hate the stupid pyramid of doom here
        match query_discogs(&client, &url) {
            Ok(text) => {
                let total: u64;
                let response: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                if let Value::Null = response { //guard clause to return immediately if cannot read
                    return Err(QueryError::ParseError);
                }
                let pagination = response.get("pagination").unwrap_or(&Value::Null);
                if pagination == &Value::Null {
                    return Err(QueryError::ParseError);
                }
                if let Value::Object(_) = pagination {
                    total = pagination.get("items").unwrap().as_u64().unwrap();
                } else {
                    return Err(QueryError::ParseError);
                }
                match parse_releases(parse.clone(), &text, false) {
                    Ok(mut releases) => {
                        master_vec.append(&mut releases);
                        if master_vec.len() as u64 == total {
                            break;
                        } else {
                            url = pagination["urls"]["next"]
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
    Ok(master_vec)
}