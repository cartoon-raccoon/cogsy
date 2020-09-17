use std::path::Path;

use std::collections::HashMap;
use serde_json::Value;
use reqwest::blocking::Client;
use chrono::{DateTime, Utc};
use crate::app::{
    {Release, Folders, Profile},
    request::*,
    database::{admin, update, purge},
};
use crate::utils;

pub fn full(username: String, token: String, from_cmd: bool, debug: bool) -> Result<(), QueryError> {
    let requester = build_client(token);
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
    if from_cmd {print!("Updating profile...")}
    let profile = match profile(&requester, username.clone()) {
        Ok(profile) => profile,
        Err(e) => {return Err(e);}
    };
    if from_cmd {print!("    Success!\nUpdating wantlist...")}
    let wantlist = match wantlist(&requester, username.clone()) {
        Ok(wantlist) => wantlist,
        Err(e) => {return Err(e);}
    };
    if from_cmd {print!("   Success!\nUpdating collection...")}
    let collection = match collection(&requester, username) {
        Ok(collection) => collection,
        Err(e) => {return Err(e);}
    };
    if from_cmd {print!(" Success!\n")}
    
    //* committing data to db
    if from_cmd {println!("Writing to database...\n")}
    match update::profile(profile) {
        Ok(_) => {},
        Err(e) => return Err(QueryError::DBWriteError(e.to_string()))
    }
    match update::wantlist(wantlist) {
        Ok(_) => {},
        Err(e) => return Err(QueryError::DBWriteError(e.to_string()))
    }
    match update::collection(collection) {
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
    if from_cmd {println!("Database update successful.")}
    Ok(())
}

pub fn profile(requester: &Client, username: String) -> Result<Profile, QueryError> {
    let profile_url = build_url(ParseType::Profile, username.clone());
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

pub fn wantlist(requester: &Client, username: String) -> Result<Vec<Release>, QueryError> {
    let wantlist_url = build_url(ParseType::Wantlist, username.clone());
    let master_wants = get_full(&requester, ParseType::Wantlist, wantlist_url)?;

    Ok(master_wants)
}

pub fn collection(requester: &Client, username: String) -> Result<Folders, QueryError> {
    //* 1a: Enumerating folders
    let initial_url = build_url(ParseType::Initial, username.clone());
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

    //*1b: Pulling each folder
    for (name, folderurl) in folders.iter() {
        let collection_url = build_url(ParseType::Folders(folderurl.clone()), username.clone());
        let releases = get_full(&requester, ParseType::Collection, collection_url)?;
        master_folders.push(name.clone(), releases);
    }
    Ok(master_folders)
}

//TODO: Make this multithreaded
//* spawn threads from this function, pass get_releases as a closure
//* might need to refactor the loop to retrieve urls from outside within collection()
//* use a common Mutex'd vector for the threads to write to
fn get_full(client: &Client, parse: ParseType, starting_url: String) -> Result<Vec<Release>, QueryError> {
    let mut url = starting_url;
    let mut master_vec: Vec<Release> = Vec::new();

    //main update loop
    //TODO: Make this nicer and handle the unwraps
    loop { //I hate the stupid pyramid of doom here
        match query_discogs(client, &url) {
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