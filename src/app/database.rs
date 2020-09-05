use crate::app::request;
use rusqlite;

/*
*the database API exposes two main modules:
*   mod update: called by the request module query() to load in data
*       fns: update_db, load_into_db
*       also exposes functions that update userid and token
*   mod query: queries from the local database
*       IMPORTANT: due to the way data is added to the screens,
*       every query must return an iterator or indexable set of iterators
?       one function per table or one big function for all transactions?
*/
pub mod update {
    pub fn update_db() {

    }
}

pub mod query {
    pub fn get_from_db() {

    }
}