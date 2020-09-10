use std::collections::HashMap;
use serde_json::Value;
use reqwest::blocking::Client;
use crate::app::{
    {Release, Folders, Profile},
    request::*,
    database::{admin, update},
};

//TODO: Add in profile and wantlist parsing
pub fn full(username: String, token: String) -> Result<(), QueryError> {
    match admin::init_db() {
        Ok(_) => {},
        Err(e) => {return Err(QueryError::DBWriteError(e.to_string()))}
    }
    match profile(username.clone(), token.clone()) {
        Ok(_) => {},
        Err(e) => {return Err(e);}
    }
    match wantlist(username.clone(), token.clone()) {
        Ok(_) => {},
        Err(e) => {return Err(e);}
    }
    match collection(username, token) {
        Ok(_) => {},
        Err(e) => {return Err(e);}
    }
    Ok(())
}

pub fn profile(username: String, token: String) -> Result<(), QueryError> {
    let requester = build_client(token);
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
        username: profile_raw["username"].as_str().unwrap().to_string(),
        real_name: profile_raw["name"].as_str().unwrap().to_string(),
        registered: profile_raw["registered"].as_str().unwrap().to_string(),
        listings: profile_raw["num_for_sale"].as_u64().unwrap() as u32,
        collection: profile_raw["num_collection"].as_u64().unwrap() as u32,
        wantlist: profile_raw["num_wantlist"].as_u64().unwrap() as u32,
        rated: profile_raw["releases_rated"].as_u64().unwrap() as u32,
        average_rating: profile_raw["rating_avg"].as_f64().unwrap(),
    };
    //committing to db
    match update::profile(master_prof) {
        Ok(_) => Ok(()),
        Err(e) => {return Err(QueryError::DBWriteError(e.to_string()))}
    }

}

pub fn wantlist(username: String, token: String) -> Result<(), QueryError> {
    let requester = build_client(token);
    let wantlist_url = build_url(ParseType::Wantlist, username.clone());
    let master_wants = get_full(&requester, ParseType::Wantlist, wantlist_url)?;

    //committing to db
    match update::wantlist(master_wants) {
        Ok(_) => Ok(()),
        Err(e) => {return Err(QueryError::DBWriteError(e.to_string()));}
    }
}

//* This should eventually return nothing, and completely write to the db
pub fn collection(username: String, token: String) -> Result<Folders, QueryError> {
    let requester = build_client(token);
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
    match update::collection(master_folders.clone()) {
        Ok(_) => {return Ok(master_folders)},
        Err(e) => {return Err(QueryError::DBWriteError(e.to_string()))},
    }
}

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