use std::path::PathBuf;
use std::fs;
use std::str::Chars;
use std::iter::Enumerate;

use directories::ProjectDirs;
use chrono::{DateTime, Utc, Local};

use crate::config::Config;
use crate::app::database::query;

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("rs", "cartoon-raccoon", "cogsy")
        .unwrap_or_else(|| panic!("Invalid home directory."))
}

pub fn config_file() -> PathBuf {
    let dirs = project_dirs();
    let mut cfgfile = PathBuf::from(dirs.config_dir());
    fs::create_dir_all(&cfgfile)
        .unwrap_or_else(|_s| panic!("Could not create config file directory"));
    cfgfile.push("config.toml");
    cfgfile
}

pub fn database_file() -> PathBuf {
    let dirs = project_dirs();
    let mut datafile = PathBuf::from(dirs.data_dir());
    fs::create_dir_all(&datafile)
        .unwrap_or_else(|_s| panic!("Could not create data file directory"));
    datafile.push("cogsy_data.db");
    datafile
}

pub fn usernames_match() -> bool {
    Config::load().user.username == query::profile().unwrap().username
}

pub fn get_utc_now() -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        Local::now().naive_utc(),
        Utc
    )
}

// Everything from this line onwards is probably some
// of the greatest code I've ever written. /s

pub enum FormatToken<'a> {
    Artist,
    Title,
    Id,
    Year,
    Labels,
    Formats,
    DateAdded,
    RawStr(&'a str),
    Unknown,
}

pub struct FormatTokenizer<'a> {
    prev: usize,
    inner: &'a str,
    chars: Enumerate<Chars<'a>>,
    buf: String,
    in_token: bool,
}

impl<'a> FormatTokenizer<'a> {
    pub fn new(format: &'a str) -> Self {
        Self {
            prev: 0,
            inner: format,
            chars: format.chars().enumerate(),
            buf: String::new(),
            in_token: false,
        }
    }
}

impl<'a> Iterator for FormatTokenizer<'a> {
    type Item = FormatToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {

        use FormatToken::*;

        while let Some((idx, c)) = self.chars.next() {
            match c {
                '{'  => {
                    self.in_token = true;
                    // if self.buf.is_empty() {
                    //     continue
                    // }
                    self.buf.clear();
                    return Some(RawStr(if let Some(r) = self.inner.get(self.prev..idx) {
                        r
                    } else {
                        ""
                    }))
                }
                '}'  => {
                    self.in_token = false;
                    let mut ret = Unknown;
                    match self.buf.as_str() {
                        "artist" => {ret = Artist}
                        "title" => {ret = Title}
                        "id" => {ret = Id}
                        "year" => {ret = Year}
                        "labels" => {ret = Labels}
                        "formats" => {ret = Formats}
                        "date" => {ret = DateAdded}
                        _ => {}
                    }

                    self.buf.clear();
                    self.prev = idx + 1;

                    return Some(ret);
                }
                '\\' => {
                    if let Some(c) = self.chars.next() {
                        self.buf.push(c.1)
                    }
                    continue
                }
                c => {
                    self.buf.push(c)
                }
            }
        }
        None
    }
}