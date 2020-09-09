/*
*the database API exposes two main modules:
*   mod update: called by the request module query() to load in data
*       also exposes functions that update userid and token
*       and log listening history
*   mod query: queries from the local database
*       every query returns an iterator or indexable set of iterators
*       wrapped in a custom type (Folders, Release)
*/
pub mod admin {
    use std::error::Error;
    use rusqlite::{
        Connection,
        NO_PARAMS,
    };

    //database initialization
    //this should only be called on first time startup
    //or when the database has been purged
    pub fn init_db(folderlist: Vec<String>) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open("cogsy_data.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profile (
                username TEXT PRIMARY KEY,
                real_name TEXT,
                registered BOOLEAN,
                listings INTEGER,
                collection INTEGER,
                wantlist INTEGER,
                rated INTEGER,
                average_rating REAL
            )",
            NO_PARAMS
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS wantlist (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT,
                year INTEGER,
                labels BLOB,
                formats BLOB,
                date_added TEXT
            )",
            NO_PARAMS
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS folders (
                name TEXT PRIMARY KEY
            )",
            NO_PARAMS
        )?;

        for foldername in folderlist {
            let sqlcommand = format!("CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT,
                year INTEGER,
                labels BLOB,
                formats BLOB,
                date_added TEXT
            )", foldername);

            conn.execute(&sqlcommand, NO_PARAMS)?;
        }
        Ok(())
    }
}

pub mod update {
    use std::error::Error;
    use rusqlite::Connection;
    use crate::app::{
        Release, 
        Folders, 
        Profile
    };

    pub fn profile(profile: Profile) -> Result<(), ()> {
        let conn = Connection::open("cogsy_data.db");
        Ok(())
    }

    pub fn collection(collection: Folders) -> Result<usize, Box<dyn Error>> {
        Ok(0)
    }

    pub fn wantlist(wantlist: Vec<Release>) -> Result<usize, Box<dyn Error>> {
        Ok(0)
    }
}

pub mod query {
    use rusqlite;
    use crate::app::{
        Release, 
        Folders, 
        Profile
    };

    pub fn profile() {

    }

    pub fn collection() -> Folders {
        Folders::new()
    }

    pub fn wantlist() -> Vec<Release> {
        Vec::new()
    }

    //returns a vec of releases to support multiple results
    pub fn release(title: String) -> Vec<Release> {
        Vec::new()
    }
}