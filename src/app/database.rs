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

pub mod admin {
    use std::error::Error;
    use rusqlite::{
        Connection,
        NO_PARAMS,
    };
    use crate::utils;

    pub fn init_db() -> Result<(), Box<dyn Error>> {
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
                id INTEGER PRIMARY KEY,
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
                title TEXT,
            )",
            NO_PARAMS
        )?;

        Ok(())
    }

    pub fn init_folder(foldername: String, conn: &Connection) 
        -> Result <(), Box<dyn Error>> {
        let sqlcommand = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY,
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
    pub fn check_integrity() -> bool {
        match Connection::open(utils::database_file()) {
            Ok(conn) => {
                match conn.prepare("SELECT * FROM profile;") {
                    Ok(_) => {},
                    Err(_) => return false
                }
                match conn.prepare("SELECT * FROM wantlist;") {
                    Ok(_) => {},
                    Err(_) => return false
                }
                match conn.prepare("SELECT * FROM listenlog;") {
                    Ok(_) => {},
                    Err(_) => return false
                }
                match conn.prepare("SELECT * FROM folders;") {
                    Ok(mut stmt) => {
                        let collection_check = stmt.query_map(
                            NO_PARAMS, 
                            |row| row.get(0)).unwrap();
                        let mut folder_names: Vec<String> = Vec::new();
                        for folder in collection_check {
                            folder_names.push(folder.unwrap());
                        }
                        for folder in folder_names {
                            let sqlcommand = format!("SELECT * FROM {}", folder);
                            match conn.query_row(&sqlcommand, NO_PARAMS, |_row| Ok(0)) {
                                Ok(_) => {},
                                Err(_) => return false
                            }
                        }
                    }
                    Err(_) => return false
                }
                true
            },
            Err(_) => return false
        }
    }
}

pub mod update {
    use std::error::Error;
    use rusqlite::{
        Connection,
    };
    use crate::utils;
    use crate::app::{
        Release, 
        Folders, 
        Profile,
        ListenLogEntry,
    };
    use super::{
        admin,
        purge,
    };

    pub fn profile(profile: Profile) -> Result<(), Box<dyn Error>> {
        purge::table("profile")?;
        let conn = Connection::open(utils::database_file())?;
        conn.execute("INSERT INTO profile 
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
            &profile.wantlist.to_string(),
            &profile.collection.to_string(),
            &profile.rated.to_string(),
            &profile.average_rating.to_string()
        ])?;
        Ok(())
    }

    pub fn collection(mut collection: Folders) -> Result<(), Box<dyn Error>> {
        purge::folders()?;
        let conn = Connection::open(utils::database_file())?;
        for (name, folder) in collection.contents.iter_mut() {
            let mut sanitized_name = name.clone();
            sanitized_name.push_str("_");
            conn.execute(
                "INSERT INTO folders (name) VALUES (?1)",
                &[&sanitized_name]
            )?;
            admin::init_folder(sanitized_name.clone(), &conn)?;
            add_releases(&conn, &sanitized_name, folder)?;
        }
        Ok(())
    }

    pub fn wantlist(mut wantlist: Vec<Release>) -> Result<(), Box<dyn Error>> {
        purge::table("wantlist")?;
        let conn = Connection::open(utils::database_file())?;
        add_releases(&conn, "wantlist", &mut wantlist)?;
        Ok(())
    }

    #[allow(dead_code)] //suppressing warnings for now
    pub fn listenlog(entry: ListenLogEntry) -> Result<(), Box<dyn Error>> {
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
                entry.title,
            ]
        )?;
        Ok(())
    }

    fn add_releases(conn: &Connection, foldername: &str, folder: &mut Vec<Release>)
        -> Result<(), Box<dyn Error>> {
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

            let mut stmt = conn.prepare(
                &format!("INSERT INTO {}
                (id,
                title,
                artist,
                year,
                labels,
                formats,
                date_added) VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7);", 
                foldername)
            )?;
            stmt.execute(
                &[
                    &release.id.to_string(),
                    &release.title,
                    &release.artist,
                    &release.year.to_string(),
                    &stringified_labels,
                    &stringified_formats,
                    &release.date_added.to_rfc3339(),
                ]
            )?;
        }
        Ok(())
    }
}

pub mod query {
    use std::error::Error;
    use chrono::{
        DateTime,
        Utc,
    };
    use rusqlite::{
        Connection,
        Statement,
        NO_PARAMS,
    };
    use crate::app::{
        Release, 
        Folders, 
        Profile,
        ListenLog,
    };
    use crate::utils;

    /*
    profile(), collection() and wantlist() are called when the app starts
    they must not fail, so they don't have to return a result
    and will panic if they do fail
    ideally, admin::check() should properly ensure database integrity before starting the app.
    however, it cannot protect against the contents of the table itself.
    */
    pub enum QueryType {
        Collection,
        Wantlist,
    }

    pub fn profile() -> Result<Profile, Box<dyn Error>> {
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

    pub fn collection() -> Result<Folders, Box<dyn Error>> {
        let conn = Connection::open(utils::database_file())?;

        let mut folder_names: Vec<String> = Vec::new();
        let mut stmt = conn.prepare("SELECT name FROM folders;")?;
        let folderquery = stmt.query_map(NO_PARAMS, |row| row.get(0))?;
        for folder in folderquery {
            folder_names.push(folder?);
        }

        let mut folders = Folders::new();

        for mut name in folder_names {
            let mut stmt = conn.prepare(&format!("SELECT * FROM {}", name))?;
            let folder = get_releases(&mut stmt)?;
            name.pop().unwrap();
            folders.push(name, folder);
        }
        Ok(folders)
    }

    pub fn wantlist() -> Result<Vec<Release>, Box<dyn Error>> {
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare("SELECT * FROM wantlist;")?;
        let wantlist = get_releases(&mut stmt)?;
        Ok(wantlist)
    }

    //returns a vec of releases to support multiple results
    //TODO: Escape single apostrophes in search terms
    pub fn release(query: String, querytype: QueryType) -> Result<Vec<Release>, Box<dyn Error>> {
        let table_to_query: String;
        match querytype {
            QueryType::Collection => {
                table_to_query = "All_".to_string();
            }
            QueryType::Wantlist => {
                table_to_query = "wantlist".to_string();
            }
        }
        let conn = Connection::open(utils::database_file())?;
        let mut stmt = conn.prepare(&format!(
            "SELECT * FROM {} WHERE title LIKE '%{}%'",
            table_to_query, query
        ))?;
        let results = get_releases(&mut stmt)?;
        
        Ok(results)
    }

    pub fn listenlog() -> Result<ListenLog, Box<dyn Error>> {
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

    fn get_releases(stmt: &mut Statement) -> Result<Vec<Release>, Box<dyn Error>> {
        let mut folder: Vec<Release> = Vec::new();

            let contents = stmt.query_map(NO_PARAMS, |row| {
                let labels_raw: String = row.get(4)?;
                let formats_raw: String = row.get(5)?;

                let labels = labels_raw.as_str()
                    .split(':')
                    .map(|s| s.to_string())
                    .collect();
                let formats = formats_raw.as_str()
                    .split(':')
                    .map(|s| s.to_string())
                    .collect();
                
                Ok(Release {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    year: row.get(3)?,
                    labels: labels,
                    formats: formats,
                    //TODO: Handle the unwrap
                    date_added: row.get(6)?,
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
    use crate::utils;

    pub fn folders() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(utils::database_file())?;

        let mut folder_names: Vec<String> = Vec::new();
        let mut stmt = conn.prepare("SELECT name FROM folders;")?;
        let folderquery = stmt.query_map(NO_PARAMS, |row| row.get(0))?;
        for folder in folderquery {
            folder_names.push(folder?);
        }
        for name in folder_names {
            let sqlcommand = format!("DROP TABLE {}", name);
            match conn.execute(&sqlcommand, NO_PARAMS) {
                Ok(_) => {},
                Err(_) => {}
            }
        }
        table("folders")?;
        Ok(())
    }

    //* DO NOT CALL THIS ON THE FOLDERS TABLE OUTSIDE OF PURGE::FOLDERS
    //* YOU **WILL** GET ORPHAN TABLES
    pub fn table(tablename: &str) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open(utils::database_file())?;
        let sqlcommand = format!("DELETE FROM {}", tablename);

        match conn.execute(&sqlcommand, NO_PARAMS) {
            Ok(_) => Ok(()),
            Err(_) => Ok(())
        }
    }

    //* This will have to be called if orphan folders are detected.
    pub fn complete() -> Result<(), Box<dyn Error>> {
        //i'd rather systematically drop tables in the folder, but this will do for now
        fs::remove_file(utils::database_file())?;
        Ok(()) //You just deleted your entire database. Congrats.
    }
}