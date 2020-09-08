/*
*the database API exposes two main modules:
*   mod update: called by the request module query() to load in data
*       fns: update_db, load_into_db
*       also exposes functions that update userid and token
*   mod query: queries from the local database
*       IMPORTANT: due to the way data is added to the screens,
*       every query must return an iterator or indexable set of iterators
*/
pub mod update {
    use std::error::Error;
    use rusqlite;
    use crate::app::{
        Release, 
        Folders, 
        Profile
    };

    pub fn profile(profile: Profile) -> Result<usize, Box<dyn Error>> {
        Ok(0)
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