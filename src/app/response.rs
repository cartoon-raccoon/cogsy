//* IMPORTANT: This file is deprecated and has no use at this point.
//* I'm just keeping this around in case I ever need it.

use serde::Deserialize;

//sub-structs of Information
#[derive(Debug, Deserialize)]
pub struct Format {
    name: String,
    qty: String,
    descriptions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    name: String,
    catno: String,
    entity_type: String,
    entity_type_name: String,
    id: u32,
    resource_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    name: String,
    anv: String,
    join: String,
    tracks: String,
    id: u64,
    resource_url: String,
}

//sub-struct of Release
#[derive(Debug, Deserialize)]
pub struct Information {
    id: u64,
    master_id: u64,
    master_url: String,
    resource_url: String,
    thumb: String,
    cover_image: String,
    title: String,
    year: u32,
    formats: Vec<Format>,
    labels: Vec<Label>,
    artists: Vec<Artist>,
    genres: Vec<String>,
    styles: Vec<String>,
}

//sub-struct of Release
#[derive(Debug, Deserialize)]
pub struct Notes {
    field_id: u32,
    value: String,
}

//main struct
#[derive(Debug, Deserialize)]
pub struct Release {
    id: u64,
    instance_id: u64,
    date_added: String,
    rating: u8,
    basic_information: Information,
    folder_id: u32,
    notes: Vec<Notes>,
}

//sub-struct of Pagination
#[derive( Debug, Deserialize)]
pub struct UrlList {
    // first: String,
    // prev: String,
    // next: String,
    // last: String,
}


//main struct
#[derive(Debug, Deserialize)]
pub struct Pagination {
    page: u32,
    pages: u32,
    per_page: u32,
    items: u32,
    urls: UrlList,
}

//ons struct to rule them all
#[derive(Debug, Deserialize)]
pub struct Response {
    pagination: Pagination,
    releases: Vec<Release>,
}

impl Response {

}
