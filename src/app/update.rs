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
use indicatif::{MultiProgress, ProgressBar};

use crate::app::{
    {Release, Folders, Profile},
    database::{admin, update, purge},
    message::{Message, MessageKind},
};
pub use crate::app::request::*;
use crate::utils;

pub fn full(username: &str, token: &str, from_cmd: bool, verbose: bool) -> Result<(), UpdateError> {
    if Path::new(&utils::database_file()).exists() {
        match admin::check_integrity() {
            Ok(()) => {},
            Err(e) => {
                if from_cmd {
                    println!("Error on integrity check: {}", e);
                    println!("Database integrity check failed, purging and refreshing now.")
                }
                match purge::complete() {
                    Ok(()) => {},
                    Err(e) => {return Err(UpdateError::DBWriteError(e.to_string()))}
                }
            }
        }
    }
    
    admin::init_db()?;
    
    //* pulling data from Discogs
    let requester = build_client(&token);
    
    if from_cmd {print!("Updating profile..."); io::stdout().flush().unwrap();}
    let profile = get_profile(&requester, username)?;
    if from_cmd {
        print!("{}", Message::set("     Success!", MessageKind::Success));
        if verbose {
            println!("\nUpdating wantlist...")
        } else {
            print!("\nUpdating wantlist...");
            io::stdout().flush().unwrap();
        }
    }

    //Spawning a synchronous thread to catch panics when parsing json
    let owned_uname = username.to_string();
    let req_clone = requester.clone();
    let wantlist = match thread::spawn( move || -> Result<Vec<Release>, UpdateError> {
        get_wantlist(req_clone, owned_uname, from_cmd, verbose)
    }).join() { //thread panics are caught here instead of crashing the entire app
        Ok(wantlist) => wantlist?,
        Err(_) => {return Err(UpdateError::ThreadPanicError);}
    };

    if from_cmd {
        if verbose {
            println!("{}", Message::success("Success!"));
            println!("\nUpdating collection...")
        } else {
            print!("{}", Message::set("    Success!", MessageKind::Success));
            print!("\nUpdating collection...");
            io::stdout().flush().unwrap();
        }
    }
    //threads are spawned from within the function
    let collection = get_collection(requester, username, from_cmd, verbose)?;

    if from_cmd {
        if verbose {
            println!("{}", Message::success("Success!"));
        } else {
            print!("{}", Message::set("  Success!", MessageKind::Success));
        }
    }
    
    //* committing data to db
    let mut dbhandle = update::DBHandle::new()?;

    if from_cmd {println!("\nWriting to database...\n")}

    dbhandle.update_profile(profile)?;
    dbhandle.update_wantlist(wantlist)?;
    dbhandle.update_collection(collection)?;

    //* final integrity check
    admin::check_integrity()?;
    
    Ok(())
}

//todo: pretty output
pub fn profile(username: &str, token: &str, from_cmd: bool) -> Result<(), UpdateError> {
    admin::init_db()?;

    //* pulling data from Discogs
    let requester = build_client(&token);

    if from_cmd {println!("Updating profile...")}
    let profile = get_profile(&requester, username)?;

    //* committing data to db
    let mut dbhandle = update::DBHandle::new()?;

    if from_cmd {println!("Writing to database..")}

    dbhandle.update_profile(profile)?;

    //* final integrity check
    Ok(admin::check_integrity()?)
}

pub fn wantlist(username: &str, token: &str, from_cmd: bool, verbose: bool) -> Result<(), UpdateError> {
    admin::init_db()?;

    //* pulling data from Discogs
    let requester = build_client(&token);

    if from_cmd {println!("Updating wantlist...")}

    //Spawning a synchronous thread to catch panics when parsing json
    let owned_uname = username.to_string();
    let req_clone = requester.clone();
    let wantlist = match thread::spawn( move || -> Result<Vec<Release>, UpdateError> {
        get_wantlist(req_clone, owned_uname, from_cmd, verbose)
    }).join() { //thread panics are caught here instead of crashing the entire app
        Ok(wantlist) => wantlist?,
        Err(_) => {return Err(UpdateError::ThreadPanicError);}
    };

    //* committing data to db
    let mut dbhandle = update::DBHandle::new()?;

    if from_cmd {println!("Writing to database...")}

    dbhandle.update_wantlist(wantlist)?;

    //* final integrity check
    Ok(admin::check_integrity()?)
}

pub fn collection(username: &str, token: &str, from_cmd: bool, verbose: bool) -> Result<(), UpdateError> {
    admin::init_db()?;

    //* pulling data from Discogs
    let requester = build_client(&token);

    if from_cmd {println!("Updating collection...")}

    //threads are spawned from within the function
    let collection = get_collection(requester, username, from_cmd, verbose)?;

    //* committing data to db
    let mut dbhandle = update::DBHandle::new()?;

    if from_cmd {println!("Writing to database...")}

    dbhandle.update_collection(collection)?;

    //* final integrity check
    Ok(admin::check_integrity()?)
}

fn get_profile(requester: &Client, username: &str) -> Result<Profile, UpdateError> {
    let profile_url = build_url(ParseType::Profile, username.to_string());
    let master_prof: Profile;

    //pulling profile and deserialization
    let response = query_discogs(&requester, &profile_url)?;
    let profile_raw: Value = serde_json::from_str(&response)
                .unwrap_or(Value::Null);
    if let Value::Null = profile_raw {
        return Err(UpdateError::JSONParseError)
    }
    master_prof = Profile {
        username: profile_raw["username"]
            .as_str().ok_or(UpdateError::JSONParseError)?.to_string(),
        real_name: profile_raw["name"]
            .as_str().ok_or(UpdateError::JSONParseError)?.to_string(),
        registered: DateTime::<Utc>::from_utc(
            DateTime::parse_from_rfc3339(
            profile_raw["registered"]
            .as_str().ok_or(UpdateError::JSONParseError)?
            ).unwrap().naive_utc(), Utc
        ),
        listings: profile_raw["num_for_sale"]
            .as_u64().ok_or(UpdateError::JSONParseError)? as u32,
        collection: profile_raw["num_collection"]
            .as_u64().ok_or(UpdateError::JSONParseError)? as u32,
        wantlist: profile_raw["num_wantlist"]
            .as_u64().ok_or(UpdateError::JSONParseError)? as u32,
        rated: profile_raw["releases_rated"]
            .as_u64().ok_or(UpdateError::JSONParseError)? as u32,
        average_rating: profile_raw["rating_avg"]
            .as_f64().ok_or(UpdateError::JSONParseError)? as f64,
    };
    Ok(master_prof)
}

fn get_wantlist(requester: Client, username: String, c: bool, v: bool) -> Result<Vec<Release>, UpdateError> {
    let wantlist_url = build_url(ParseType::Wantlist, username);
    let pgb = if c {ProgressBar::new(60)} else {ProgressBar::hidden()};
    let master_wants = get_full(
        Arc::new(requester), 
        ParseType::Wantlist, 
        wantlist_url, pgb, v, 
        "Wantlist".into()
    )?;

    Ok(master_wants)
}

fn get_collection(requester: Client, username: &str, c: bool, v: bool) -> Result<Folders, UpdateError> {
    //* 1a: Enumerating folders
    let initial_url = build_url(ParseType::Initial, username.to_string());
    let folders_raw = query_discogs(&requester, &initial_url)?;
    let mut folders = HashMap::<String, String>::new();
    let total_prog = MultiProgress::new();

    let to_deserialize: Value = serde_json::from_str(&folders_raw)
        .unwrap_or(Value::Null);
    if let Value::Null = to_deserialize {
        return Err(UpdateError::JSONParseError);
    }
    let folders_raw = to_deserialize.get("folders");
    if let Some(Value::Array(result)) = folders_raw {
        let folderlist = result;

        for raw in folderlist.iter() {
            let foldername = raw.get("name")
                .ok_or(UpdateError::JSONParseError)?.as_str()
                .ok_or(UpdateError::JSONParseError)?.to_string();
            let folderurl = raw.get("resource_url")
                .ok_or(UpdateError::JSONParseError)?.as_str()
                .ok_or(UpdateError::JSONParseError)?.to_string();
            folders.insert(foldername, folderurl);
        } 
    } else {return Err(UpdateError::JSONParseError);}

    let mut master_folders: Folders = Folders::new();
    let mut threads = Vec::new();
    let requester = Arc::new(requester);
    // let (tx, rx) = mpsc::channel();

    //*1b: Pulling each folder
    for (name, folderurl) in folders {
        let owned_uname = username.to_string();
        let req_clone = requester.clone();
        let pb = if c { ProgressBar::new(60)} else {ProgressBar::hidden()};
        total_prog.add(pb.clone());
        threads.push(thread::spawn( move || -> Result<(String, Vec<Release>), UpdateError> {
            let collection_url = build_url(ParseType::Folders(folderurl), owned_uname);
            let releases = get_full(req_clone, ParseType::Collection, collection_url, pb, v, name.clone())?;
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
    total_prog.join().unwrap();
    Ok(master_folders)
}

fn get_full(
    client: Arc<Client>, 
    parse: ParseType, 
    starting_url: String, 
    _pb: ProgressBar,
    from_cli: bool,
    name: String,
) -> Result<Vec<Release>, UpdateError> {
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
            return Err(UpdateError::JSONParseError);
        }
        let pagination = response.get("pagination").unwrap_or(&Value::Null);
        if pagination == &Value::Null {
            return Err(UpdateError::JSONParseError);
        }
        if let Value::Object(_) = pagination {
            total = pagination.get("items")
                .ok_or(UpdateError::JSONParseError)?
                .as_u64()
                .ok_or(UpdateError::JSONParseError)?;
        } else {
            return Err(UpdateError::JSONParseError);
        }
        let mut releases = parse_releases(parse.clone(), &text, false, from_cli, &name)?; 
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
                                .ok_or(UpdateError::JSONParseError)?
                                .to_string();
        }
    }
    Ok(master_vec)
}