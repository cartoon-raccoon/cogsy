pub mod impl_app;
pub mod database;
pub mod request;
pub mod message;
pub mod response;
pub mod update;

use std::collections::HashMap;

use message::Message;
use crate::collection::Collection;

//#[derive(Debug, Clone)]
pub struct App {
    pub user_id: String,
    pub token: String,
    pub message: Message,
    pub collection: Collection,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub username: String,
    pub real_name: String,
    pub registered: String,
    pub listings: u64,
    pub collection: u64,
    pub wantlist: u64,
    pub rated: u64,
    pub average_rating: f64,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub year: u64,
    pub labels: Vec<String>,
    pub formats: Vec<String>,
    pub date_added: String,
}

#[derive(Debug, Clone)]
pub struct Folders {
    pub contents: HashMap<String, Vec<Release>>,
}