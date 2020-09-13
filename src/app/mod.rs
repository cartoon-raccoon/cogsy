pub mod impl_app;
pub mod database;
pub mod request;
pub mod message;
pub mod response;
pub mod update;

use std::collections::HashMap;

use chrono::{
    DateTime,
    FixedOffset,
};
use message::Message;
use crate::collection::Collection;

#[derive(Debug, Clone)]
pub struct App {
    pub user_id: String,
    pub token: String,
    pub message: Message,
    pub collection: Collection,
    pub modified: bool,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub username: String,
    pub real_name: String,
    pub registered: DateTime<FixedOffset>,
    pub listings: u32,
    pub collection: u32,
    pub wantlist: u32,
    pub rated: u32,
    pub average_rating: f64,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub labels: Vec<String>,
    pub formats: Vec<String>,
    pub date_added: DateTime<FixedOffset>,
}

#[derive(Debug, Clone)]
pub struct Folders {
    pub contents: HashMap<String, Vec<Release>>,
}