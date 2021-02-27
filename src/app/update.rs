use std::{
    io::{self, Write},
    path::Path,
    thread,
    sync::Arc,
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

pub fn full(username: &str, token: &str, from_cmd: bool, debug: bool) -> Result<(), UpdateError> {
    if Path::new(&utils::database_file()).exists() {
        match admin::check_integrity() {
            true => {},
            false => {
                if debug {println!("Database integrity check failed, purging and refreshing now.")}
                match purge::complete() {
                    Ok(()) => {},
                    Err(e) => {return Err(UpdateError::DBWriteError(e.to_string()))}
                }
            }
        }
    }
    match admin::init_db() {
        Ok(_) => {},
        Err(e) => {return Err(UpdateError::DBWriteError(e.to_string()))}
    }
    
    //* pulling data from Discogs
    let requester = build_client(&token);
    
    if from_cmd {print!("Updating profile..."); io::stdout().flush().unwrap();}
    let profile = match profile(&requester, username) {
        Ok(profile) => profile,
        Err(e) => {return Err(e);}
    };
    if from_cmd {
        print!("{}", Message::set("     Success!", MessageKind::Success));
        print!("\nUpdating wantlist...");
        io::stdout().flush().unwrap();
    }

    //Spawning a synchronous thread to catch panics when parsing json
    let owned_uname = username.to_string();
    let req_clone = requester.clone();
    let wantlist = match thread::spawn( move || -> Result<Vec<Release>, UpdateError> {
        match wantlist(req_clone, owned_uname) {
            Ok(wantlist) => Ok(wantlist),
            Err(e) => Err(e)
        }
    }).join() { //thread panics are caught here instead of crashing the entire app
        Ok(wantlist) => wantlist?,
        Err(_) => {return Err(UpdateError::ThreadPanicError);}
    };

    if from_cmd {
        print!("{}", Message::set("    Success!", MessageKind::Success));
        print!("\nUpdating collection...");
        io::stdout().flush().unwrap();
    }
    //threads are spawned from within the function
    let collection = match collection(requester, username) {
        Ok(collection) => collection,
        Err(e) => {return Err(e);}
    };
    if from_cmd {print!("{}", Message::set("  Success!", MessageKind::Success));}
    
    //* committing data to db
    let mut dbhandle = match update::DBHandle::new() {
        Ok(handle) => handle,
        Err(e) => {return Err(UpdateError::DBWriteError(e.to_string()));}
    };

    if from_cmd {println!("\nWriting to database...\n")}
    match dbhandle.update_profile(profile) {
        Ok(_) => {},
        Err(e) => return Err(UpdateError::DBWriteError(e.to_string()))
    }
    match dbhandle.update_wantlist(wantlist) {
        Ok(_) => {},
        Err(e) => return Err(UpdateError::DBWriteError(e.to_string()))
    }
    match dbhandle.update_collection(collection) {
        Ok(_) => {},
        Err(e) => return Err(UpdateError::DBWriteError(e.to_string())),
    }

    //* final integrity check
    match admin::check_integrity() {
        true => {},
        false => {
            let errormsg = String::from("Integrity check failed");
            return Err(UpdateError::DBWriteError(errormsg))
        }
    }
    Ok(())
}

fn profile(requester: &Client, username: &str) -> Result<Profile, UpdateError> {
    let profile_url = build_url(ParseType::Profile, username.to_string());
    let master_prof: Profile;

    //pulling profile and deserialization
    let response = query_discogs(&requester, &profile_url)?;
    let profile_raw: Value = serde_json::from_str(&response)
                .unwrap_or(Value::Null);
    if let Value::Null = profile_raw {
        return Err(UpdateError::ParseError)
    }
    master_prof = Profile {
        username: profile_raw["username"]
            .as_str().ok_or(UpdateError::ParseError)?.to_string(),
        real_name: profile_raw["name"]
            .as_str().ok_or(UpdateError::ParseError)?.to_string(),
        registered: DateTime::<Utc>::from_utc(
            DateTime::parse_from_rfc3339(
            profile_raw["registered"]
            .as_str().ok_or(UpdateError::ParseError)?
            ).unwrap().naive_utc(), Utc
        ),
        listings: profile_raw["num_for_sale"]
            .as_u64().ok_or(UpdateError::ParseError)? as u32,
        collection: profile_raw["num_collection"]
            .as_u64().ok_or(UpdateError::ParseError)? as u32,
        wantlist: profile_raw["num_wantlist"]
            .as_u64().ok_or(UpdateError::ParseError)? as u32,
        rated: profile_raw["releases_rated"]
            .as_u64().ok_or(UpdateError::ParseError)? as u32,
        average_rating: profile_raw["rating_avg"]
            .as_f64().ok_or(UpdateError::ParseError)? as f64,
    };
    Ok(master_prof)
}

fn wantlist(requester: Client, username: String) -> Result<Vec<Release>, UpdateError> {
    let wantlist_url = build_url(ParseType::Wantlist, username);
    let master_wants = get_full(Arc::new(requester), ParseType::Wantlist, wantlist_url)?;

    Ok(master_wants)
}

fn collection(requester: Client, username: &str) -> Result<Folders, UpdateError> {
    //* 1a: Enumerating folders
    let initial_url = build_url(ParseType::Initial, username.to_string());
    let folders_raw = query_discogs(&requester, &initial_url)?;
    let mut folders = HashMap::<String, String>::new();

    let to_deserialize: Value = serde_json::from_str(&folders_raw)
        .unwrap_or(Value::Null);
    if let Value::Null = to_deserialize {
        return Err(UpdateError::ParseError);
    }
    let folders_raw = to_deserialize.get("folders");
    if let Some(Value::Array(result)) = folders_raw {
        let folderlist = result;

        for raw in folderlist.iter() {
            let foldername = raw.get("name")
                .ok_or(UpdateError::ParseError)?.as_str()
                .ok_or(UpdateError::ParseError)?.to_string();
            let folderurl = raw.get("resource_url")
                .ok_or(UpdateError::ParseError)?.as_str()
                .ok_or(UpdateError::ParseError)?.to_string();
            folders.insert(foldername, folderurl);
        } 
    } else {return Err(UpdateError::ParseError);}

    let mut master_folders: Folders = Folders::new();
    let mut threads = Vec::new();
    let requester = Arc::new(requester);
    // let (tx, rx) = mpsc::channel();

    //*1b: Pulling each folder
    for (name, folderurl) in folders {
        let owned_uname = username.to_string();
        let req_clone = requester.clone();
        threads.push(thread::spawn( move || -> Result<(String, Vec<Release>), UpdateError> {
            let collection_url = build_url(ParseType::Folders(folderurl), owned_uname);
            let releases = get_full(req_clone, ParseType::Collection, collection_url)?;
            Ok((name, releases))
        }));
    }
    for thread in threads {
        let (name, releases) = match thread.join() {
            Ok(result) => result?,
            Err(_) => {return Err(UpdateError::ThreadPanicError);}
        };
        master_folders.push(name, releases);
    };
    Ok(master_folders)
}

fn get_full(client: Arc<Client>, parse: ParseType, starting_url: String) -> Result<Vec<Release>, UpdateError> {
    use std::rc::Rc;

    let mut url = starting_url;
    let mut master_vec: Vec<Release> = Vec::new();
    let parse = Rc::new(parse);

    //main update loop
    loop {
        let text =  query_discogs(&client, &url)?;
        let total: u64;
        let response: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if let Value::Null = response { //guard clause to return immediately if cannot read
            return Err(UpdateError::ParseError);
        }
        let pagination = response.get("pagination").unwrap_or(&Value::Null);
        if pagination == &Value::Null {
            return Err(UpdateError::ParseError);
        }
        if let Value::Object(_) = pagination {
            total = pagination.get("items")
                .ok_or(UpdateError::ParseError)?
                .as_u64()
                .ok_or(UpdateError::ParseError)?;
        } else {
            return Err(UpdateError::ParseError);
        }
        let mut releases = parse_releases(parse.clone(), &text, false)?; 
        master_vec.append(&mut releases);
        if master_vec.len() as u64 == total {
            break;
        } else {
            //* Quite risky as wantlist is not paginated
            //* and its json will not have this field
            //* Should be safe as execution should never reach this branch
            //* on wantlist update, but is still risky nonetheless
            url = pagination["urls"]["next"]
                                .as_str()
                                .ok_or(UpdateError::ParseError)?
                                .to_string();
        }
    }
    Ok(master_vec)
}