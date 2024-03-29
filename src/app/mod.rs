pub mod impl_app;
pub mod database;
pub mod csv;
pub mod request;
pub mod message;
pub mod response;
pub mod update;
pub mod cli;

use std::collections::BTreeMap;
use std::hash::Hash;
use std::fmt;

use chrono::{
    DateTime,
    Utc,
};
use message::Message;

use crate::CONFIG;
use crate::collection::Collection;
use crate::config::Appearance;
use crate::utils::{FormatTokenizer, FormatToken};

#[derive(Debug, Clone)]
pub struct App {
    pub user_id: String,
    pub token: String,
    pub message: Message,
    pub collection: Collection,
    pub modified: bool,
    pub appearance: Appearance,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub username: String,
    pub real_name: String,
    pub registered: DateTime<Utc>,
    pub listings: u32,
    pub collection: u32,
    pub wantlist: u32,
    pub rated: u32,
    pub average_rating: f64,
}

pub struct ListenLogEntry<'a> {
    pub id: i64,
    pub title: &'a str,
    pub time: DateTime<Utc>,
}

pub struct ListenLog { //wrapper around a BTreeMap
    pub contents: BTreeMap<DateTime<Utc>, String>
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Release {
    pub id: i64,
    pub search_string: String,
    pub title: String,
    pub artist: String,
    pub year: u32,
    pub labels: Vec<String>,
    pub formats: Vec<String>,
    pub date_added: DateTime<Utc>,
}

impl Release {
    pub fn format(&self, format_str: &str) -> String {
        use FormatToken::*;

        let mut ret = String::from("");

        let tokenizer = FormatTokenizer::new(format_str);

        for token in tokenizer {
            match token {
                Artist => {ret.push_str(&self.artist)}
                Title => {ret.push_str(&self.title)}
                Id => {ret.push_str(&self.id.to_string())}
                Year => {ret.push_str(&self.id.to_string())}
                Labels => {ret.push_str(&self.labels.join(", "))}
                Formats => {ret.push_str(&self.formats.join(", "))}
                DateAdded => {ret.push_str(&self.date_added.to_string())}
                RawStr(s) => {ret.push_str(s)}
                Unknown => {}
            }
        }

        ret
    }
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_time = self.date_added.with_timezone(&CONFIG.timezone());
        write!(f,
            "{} by {}:\nReleased: {}\nLabels: {}\nFormats: {}\nAdded: {}\n",
            self.title,
            self.artist,
            self.year,
            self.labels.join(", "),
            self.labels.join(", "),
            display_time.format("%A %d %m %Y %R"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Folders {
    pub contents: BTreeMap<String, Vec<Release>>,
}

#[allow(dead_code)]
impl Folders { //wrapper around a BTreeMap<String, Vec<Release>>
    pub fn new() -> Self {
        let new_self: 
            BTreeMap<String, Vec<Release>> = BTreeMap::new();
        Folders {
            contents: new_self,
        }
    }
    pub fn contents(&mut self) -> BTreeMap<String, Vec<Release>> {
        self.contents.clone()
    }

    pub fn pull(&mut self, name: &str) -> Option<Vec<Release>> {
        self.contents.remove(name)
    }

    pub fn push(&mut self, 
        folder: String, 
        contents: Vec<Release>) -> Option<Vec<Release>> {
        
        self.contents.insert(folder, contents)
    }

    pub fn insert(&mut self, folder: String, release: Release) {
        self.contents.entry(folder)
            .and_modify(|v| v.push(release))
            .or_insert(vec![]);
    }
}