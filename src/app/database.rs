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
    pub fn init_db() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open("cogsy_data.db")?;
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
                labels BLOB,
                formats BLOB,
                date_added TEXT
            )", foldername);

        conn.execute(&sqlcommand, NO_PARAMS)?;
        Ok(())
    }
}

pub mod update {
    use std::error::Error;
    use rusqlite::{
        Connection,
        NO_PARAMS
    };
    use crate::app::{
        Release, 
        Folders, 
        Profile
    };
    use crate::app::database::admin;

    pub fn profile(profile: Profile) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open("cogsy_data.db")?;
        conn.execute("DELETE FROM profile;", NO_PARAMS)?;
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
            &profile.registered,
            &profile.listings.to_string(),
            &profile.wantlist.to_string(),
            &profile.collection.to_string(),
            &profile.rated.to_string(),
            &profile.average_rating.to_string()
        ])?;
        Ok(())
    }

    pub fn collection(mut collection: Folders) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open("cogsy_data.db")?;
        for (name, folder) in collection.contents.iter_mut() {
            let mut sanitized_name = name.clone();
            sanitized_name.push_str("_");
            admin::init_folder(sanitized_name.clone(), &conn)?;
            add_release(&conn, &sanitized_name, folder)?;
        }
        Ok(())
    }

    pub fn wantlist(mut wantlist: Vec<Release>) -> Result<(), Box<dyn Error>> {
        let conn = Connection::open("cogsy_data.db")?;
        conn.execute("DELETE FROM wantlist;", NO_PARAMS)?;
        add_release(&conn, "wantlist", &mut wantlist)?;
        Ok(())
    }

    fn add_release(conn: &Connection, foldername: &str, folder: &mut Vec<Release>)
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
                    &release.date_added,
                ]
            )?;
        }
        Ok(())
    }
}

pub mod query {
    use std::error::Error;
    use rusqlite;
    use crate::app::{
        Release, 
        Folders, 
        Profile
    };

    /*
    profile(), collection() and wantlist() are called when the app starts
    they must not fail, so they don't have to return a result
    ideally, admin::check() should properly ensure database integrity before starting the app.
    however, it cannot protect against the contents of the table itself.
    */

    pub fn profile() {

    }

    pub fn collection() -> Folders {
        Folders::new()
    }

    pub fn wantlist() -> Vec<Release> {
        Vec::new()
    }

    //returns a vec of releases to support multiple results
    pub fn release(title: String) -> Result<Vec<Release>, Box<dyn Error>> {
        Ok(Vec::new())
    }
}