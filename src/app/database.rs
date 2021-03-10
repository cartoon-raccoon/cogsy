/*
*the database API exposes four main modules:
*   mod admin: contains functions that administer the database
*       check_integrity(), init_db(), init_folders()
*       WARNING: ORPHAN TABLES ARE A DEFINITE POSSIBILITY.
*   mod update: called by the request module query() to load in data
*       also exposes functions that update userid and token
*       and log listening history
*   mod query: queries from the local database
*       every query returns an iterator or indexable set of iterators
*       wrapped in a custom type (Folders, Release)
*   mod purge: deleting stuff from the database
*       folders(): systematically deleting collection folders
*       table(): clears a specified folder
*       complete(): yeets the entire fucking database
*       (think sudo rm -rf)
*/

use crate::app::request::UpdateError;

impl From<rusqlite::Error> for UpdateError {
    fn from(error: rusqlite::Error) -> Self {
        UpdateError::DBWriteError(error.to_string())
    }
}

impl From<DBError> for UpdateError {
    fn from(error: DBError) -> Self {
        UpdateError::DBWriteError(error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DBError {
    error: String,
}

impl From<&str> for DBError {
    fn from(from: &str) -> Self {
        DBError {
            error: from.into()
        }
    }
}

impl std::error::Error for DBError {}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<rusqlite::Error> for DBError {
    fn from(error: rusqlite::Error) -> Self {
        DBError {
            error: error.to_string()
        }
    }
}

pub mod admin {
    use rusqlite::{
        Connection,
        NO_PARAMS,
    };
    use super::DBError;
    use crate::utils;

    pub fn init_db() -> Result<(), DBError> {
        let conn = Connection::open(utils::database_file())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profile (
                username TEXT PRIMARY KEY,
                real_name TEXT,
                registered TEXT,
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
                idx INTEGER PRIMARY KEY,
                hash INTEGER,
                id INTEGER,
                search_string TEXT,
                title TEXT NOT NULL,
                artist TEXT,
                year INTEGER,
                labels TEXT,
                formats TEXT,
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS listenlog (
                datetime TEXT PRIMARY KEY,
                album_id INTEGER,
                title TEXT
            )",
            NO_PARAMS
        )?;

        Ok(())
    }

    pub fn init_folder(foldername: String, conn: &Connection) 
        -> Result <(), DBError> {
        let sqlcommand = format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (
                idx INTEGER PRIMARY KEY,
                hash INTEGER,
                id INTEGER,
                search_string TEXT,
                title TEXT NOT NULL,
                artist TEXT,
                year INTEGER,
                labels TEXT,
                formats TEXT,
                date_added TEXT
            )", foldername);

        conn.execute(&sqlcommand, NO_PARAMS)?;
        Ok(())
    }

    //TODO: Account for orphan folder tables
    pub fn check_integrity() -> Result<(), DBError> {
        match Connection::open(utils::database_file()) {
            Ok(conn) => {
                match conn.prepare("SELECT * FROM profile;") {
                    Ok(_) => {},
                    Err(e) => return Err(format!("profile: {}", e).as_str().into())
                }
                match conn.prepare("SELECT * FROM wantlist;") {
                    Ok(_) => {},
                    Err(e) => return Err(format!("wantlist: {}", e).as_str().into())
                }
                match conn.prepare("SELECT * FROM listenlog;") {
                    Ok(_) => {},
                    Err(e) => return Err(format!("listenlog: {}", e).as_str().into())
                }
                match conn.prepare("SELECT * FROM folders;") {
                    Ok(mut stmt) => {
                        let collection_check = stmt.query_map(
                            NO_PARAMS, 
                            |row| row.get(0))?;
                        let mut folder_names: Vec<String> = Vec::new();
                        for folder in collection_check {
                            folder_names.push(folder.unwrap());
                        }
                        for folder in folder_names {
                            let sqlcommand = format!("SELECT * FROM \"{}\"", folder);
                            match conn.prepare(&sqlcommand) {
                                Ok(_) => {},
                                Err(e) => return Err(format!("{}: {}", folder, e).as_str().into())
                            }
                        }
                    }
                    Err(e) => return Err(format!("folders: {}", e).as_str().into())
                }
                Ok(())
            },
            Err(_) => return Err("database file does not exist".into())
        }
    }
}

pub mod update {
    use rusqlite::{
        Connection,
    };
    use std::mem;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hasher, Hash};
    use crate::utils;
    use crate::app::{
        Release, 
        Folders, 
        Profile,
        ListenLogEntry,
        request::UpdateError,
    };
    use super::{
        admin,
        purge,
        DBError,
    };

    pub struct DBHandle {
        conn: Connection,
    }

    impl DBHandle {
        pub fn new() -> Result<Self, DBError> {
            let connection = Connection::open(utils::database_file())?;
            Ok(DBHandle {
                conn: connection,
            })
        }

        pub fn update_profile(&mut self, profile: Profile) -> Result<(), UpdateError> {
            purge::table("profile")?;
            self.conn.execute("INSERT INTO profile 
            (username, 
            real_name, 
            registered, 
            listings, 
            collection, 
            wantlist,
            rated,
            average_rating) VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);", 
            &[
                &profile.username,
                &profile.real_name,
                &profile.registered.to_rfc3339(),
                &profile.listings.to_string(),
                &profile.collection.to_string(),
                &profile.wantlist.to_string(),
                &profile.rated.to_string(),
                &profile.average_rating.to_string()
            ])?;
            Ok(())
        }

        pub fn update_collection(&mut self, mut collection: Folders) -> Result<(), UpdateError> {
            purge::folders()?;
            for (name, folder) in collection.contents.iter_mut() {
                let mut sanitized_name = name.clone();
                sanitized_name.push_str("_");
                self.conn.execute(
                    "INSERT INTO folders (name) VALUES (?1)",
                    &[&sanitized_name]
                )?;
                let mut new = Vec::new();
                mem::swap(&mut new, folder);
                admin::init_folder(sanitized_name.clone(), &self.conn)?;
                add_releases(&self.conn, &sanitized_name, new)?;
            }
            Ok(())
        }

        pub fn update_wantlist(&mut self, wantlist: Vec<Release>) -> Result<(), UpdateError> {
            purge::table("wantlist")?;
            add_releases(&self.conn, "wantlist", wantlist)?;
            Ok(())
        }
    }

    pub fn listenlog(entry: ListenLogEntry) -> Result<(), DBError> {
        let conn = Connection::open(utils::database_file())?;
        conn.execute(
            "INSERT INTO listenlog
            (datetime,
            album_id,
            title) VALUES
            (?1, ?2, ?3);",
            &[
                entry.time.to_rfc3339(),
                entry.id.to_string(),
                entry.title.into(),
            ]
        )?;
        Ok(())
    }

    fn add_releases(conn: &Connection, foldername: &str, folder: Vec<Release>)
        -> Result<(), DBError> {
        let mut idx: u64 = 1;
        let mut hasher = DefaultHasher::new();
        for release in folder {
            let mut stringified_labels = String::new();
            for label in &release.labels {
                stringified_labels.push_str(&label);
                stringified_labels.push(':');
            }

            let mut stringified_formats = String::new();
            for format in &release.formats {
                stringified_formats.push_str(&format);
                stringified_formats.push(':');
            }

            stringified_labels.pop().unwrap();
            stringified_formats.pop().unwrap();

            //hashing the release
            release.hash(&mut hasher);
            let hash = hasher.finish();

            //inserting the data
            let mut stmt = conn.prepare(
                &format!("INSERT INTO \"{}\"
                (idx,
                hash,
                id,
                search_string,
                title,
                artist,
                year,
                labels,
                formats,
                date_added) VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);", 
                foldername)
            )?;
            stmt.execute(
                &[
                    &idx.to_string(),
                    &hash.to_string(),
                    &release.id.to_string(),
                    &release.search_string,
                    &release.title,
                    &release.artist,
                    &release.year.to_string(),
                    &stringified_labels,
                    &stringified_formats,
                    &release.date_added.to_rfc3339(),
                ]
            )?;
            idx += 1;
        }
        Ok(())
    }
}

pub mod query {
    use chrono::{
        DateTime,
        Utc,
    };
    use rusqlite::{
        Connection,
        Statement,
        NO_PARAMS,
    };
    use super::DBError;
    use crate::app::{
        Release, 
        Folders, 
        Profile,
        ListenLog,
    };
    use crate::utils;

    /*
    profile(), collection() and wantlist() are called when the app starts
    they must not fail, so they will panic if they do
    ideally, admin::check() should properly ensure database integrity before starting the app.
    however, it cannot protect against the contents of the table itself.
    */
    pub enum QueryType {
        Collection,
        Wantlist,
    }

    pub fn profile() -> Result<Profile, DBError> {
        let conn = Connection::open(utils::database_file())?;

        let profile = conn.query_row(
            "SELECT * FROM profile", NO_PARAMS, |row| {
                Ok(Profile {
                    username: row.get(0)?,
                    real_name: row.get(1)?,
                    registered: row.get(2)?,
                    listings: row.get(3)?,
                    collection: row.get(4)?,
                    wantlist: row.get(5)?,
                    rated: row.get(6)?,
                    average_rating: row.get(7)?,
                })
            })?;
        
        Ok(profile)
    }

    pub fn collection() -> Result<Folders, DBError> {
        let conn = Connection::open(utils::database_file())?;

        let mut folder_names: Vec<String> = Vec::new();
        let mut stmt = conn.prepare("SELECT name FROM folders;")?;
        let folderquery = stmt.query_map(NO_PARAMS, |row| row.get(0))?;
        for folder in folderquery {
            folder_names.push(folder?);
        }

        let mut folders = Folders::new();

        for mut name in folder_names {
            let mut stmt = conn.prepare(&format!("SELECT * FROM \"{}\"", name))?;
            let folder = get_releases(&mut stmt, QueryType::Collection)?;
            name.pop().unwrap();
            folders.push(name, folder);
        }
        Ok(folders)
    }

    pub fn wantlist() -> Result<Vec<Release>, DBError> {
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare("SELECT * FROM wantlist;")?;
        let wantlist = get_releases(&mut stmt, QueryType::Wantlist)?;
        Ok(wantlist)
    }

    //returns a vec of releases to support multiple results
    pub fn release(query: String, querytype: QueryType) -> Result<Vec<Release>, DBError> {
        let table_to_query: String = match querytype {
            QueryType::Collection => "All_".to_string(),
            QueryType::Wantlist => "wantlist".to_string()
        };
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare(&format!(
            "SELECT * FROM {} WHERE search_string LIKE '%{}%'",
            table_to_query, query
        ))?;
        let results = get_releases(&mut stmt, querytype)?;
        
        Ok(results)
    }

    pub fn all_titles() -> Result<Vec<String>, DBError> {
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare(
            "SELECT title FROM All_;"
        )?;
        let titles = stmt.query_map(NO_PARAMS, |row| row.get(0))?;
        let mut titlevec = Vec::<String>::with_capacity(
            size(QueryType::Collection).unwrap_or(100)
        );
        for title in titles {
            titlevec.push(title?);
        }
        Ok(titlevec)
    }

    pub fn listenlog() -> Result<ListenLog, DBError> {
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare(
            "SELECT * FROM listenlog;"
        )?;
        let results_iter = stmt.query_map(NO_PARAMS, |row| {
            let time: DateTime<Utc> = row.get(0)?;
            let title: String = row.get(2)?;
            Ok((time, title))
        })?;
        let mut listenlog = ListenLog::new();
        for entry in results_iter {
            let (time, title) = entry?;
            listenlog.push(time, title);
        }
        Ok(listenlog)
    }

    pub fn random() -> Result<Release, DBError> {
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare(
            "SELECT * FROM All_ ORDER BY RANDOM() LIMIT 1",
        )?;
        let mut selection = get_releases(&mut stmt, QueryType::Collection)?;
        Ok(selection.remove(0))
    }

    pub fn size(querytype: QueryType) -> Result<usize, DBError> {
        let conn = Connection::open(utils::database_file())?;
        let query = match querytype {
            QueryType::Collection => "collection",
            QueryType::Wantlist => "wantlist"
        };
        let size: u32 = conn.query_row(
            &format!("SELECT {} FROM profile;", query),
            NO_PARAMS,
            |row| row.get(0)
        )?;
        Ok(size as usize)
    }

    fn get_releases(stmt: &mut Statement, querytype: QueryType) -> Result<Vec<Release>, DBError> {
        let mut folder: Vec<Release> = Vec::with_capacity(size(querytype).unwrap_or(100));

            let contents = stmt.query_map(NO_PARAMS, |row| {
                let labels_raw: String = row.get(7)?;
                let formats_raw: String = row.get(8)?;

                let labels = labels_raw.as_str()
                    .split(':')
                    .map(|s| s.to_string())
                    .collect();
                let formats = formats_raw.as_str()
                    .split(':')
                    .map(|s| s.to_string())
                    .collect();
                
                Ok(Release {
                    id: row.get(2)?,
                    search_string: String::new(),
                    title: row.get(4)?,
                    artist: row.get(5)?,
                    year: row.get(6)?,
                    labels: labels,
                    formats: formats,
                    date_added: row.get(9)?,
                })
            })?;
            for release in contents {
                folder.push(release?);
            }
            Ok(folder)
    }
}

pub mod purge {
    use std::error::Error;
    use std::fs;
    use rusqlite::{
        Connection,
        NO_PARAMS
    };
    use super::DBError;
    use crate::utils;

    pub fn folders() -> Result<(), DBError> {
        let conn = Connection::open(utils::database_file())?;

        let mut folder_names: Vec<String> = Vec::new();
        let mut stmt = conn.prepare("SELECT name FROM folders;")?;
        let folderquery = stmt.query_map(NO_PARAMS, |row| row.get(0))?;
        for folder in folderquery {
            folder_names.push(folder?);
        }
        for name in folder_names {
            let sqlcommand = format!("DROP TABLE \"{}\"", name);
            conn.execute(&sqlcommand, NO_PARAMS)?;
        }
        table("folders")?;
        Ok(())
    }

    //* DO NOT CALL THIS ON THE FOLDERS TABLE OUTSIDE OF PURGE::FOLDERS
    //* YOU **WILL** GET ORPHAN TABLES
    pub fn table(tablename: &str) -> Result<(), DBError> {
        let conn = Connection::open(utils::database_file())?;
        let sqlcommand = format!("DELETE FROM \"{}\"", tablename);
        conn.execute(&sqlcommand, NO_PARAMS)?;
        Ok(())
    }

    //* This will have to be called if orphan folders are detected.
    pub fn complete() -> Result<(), Box<dyn Error>> {
        //i'd rather systematically drop tables in the folder, but this will do for now
        fs::remove_file(utils::database_file())?;
        Ok(()) //You just deleted your entire database. Congrats.
    }
}