use std::path::PathBuf;
use std::fs;

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
    Config::load().user.username 
    == 
    query::profile().unwrap().username
}

pub fn get_utc_now() -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        Local::now().naive_utc(),
        Utc
    )
}