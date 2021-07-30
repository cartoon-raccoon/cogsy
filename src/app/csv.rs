use std::path::Path;

use csv::{Reader, StringRecord};
use unidecode::unidecode;
use chrono::{DateTime, Utc};

use crate::app::{request::UpdateError, Release};
use crate::utils;
use crate::CONFIG;

macro_rules! validate {
    ($headers:expr, $idx:expr, $targ:expr) => {
        $headers.get($idx)
        .and_then(|s| if s == $targ {Some(())} else {None})
        .ok_or(UpdateError::CSVParseError)?;
    }
}

impl From<csv::Error> for UpdateError {
    fn from(error: csv::Error) -> UpdateError {
        match error.into_kind() {
            csv::ErrorKind::Io(_) => UpdateError::IOError,
            _ => UpdateError::CSVParseError,
        }
    }
}

// constants for each column's position.
const CATALOG:     usize = 0;
const ARTIST:      usize = 1;
const TITLE:       usize = 2;
const LABEL:       usize = 3;
const FORMAT:      usize = 4;
const RATING:      usize = 5;
const RELEASED:    usize = 6;
const RELEASE_ID:  usize = 7;
const COL_FOLDER:  usize = 8;
const DATE_ADDED:  usize = 9;
const MEDIA_COND:  usize = 10;
const SLEEVE_COND: usize = 11;
const COLL_NOTES:  usize = 12;

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Release>, UpdateError> {
    let mut reader = Reader::from_path(path)?;

    validate_headers(reader.headers()?)?;

    todo!()
}

fn validate_headers(headers: &StringRecord) -> Result<(), UpdateError> {
    validate!(headers, CATALOG, "Catalog#");
    validate!(headers, ARTIST, "Artist");
    validate!(headers, TITLE, "Title");
    validate!(headers, LABEL, "Label");
    validate!(headers, FORMAT, "Format");
    validate!(headers, RATING, "Rating");
    validate!(headers, RELEASED, "Released");
    validate!(headers, RELEASE_ID, "release_id");
    validate!(headers, COL_FOLDER, "CollectionFolder");
    validate!(headers, DATE_ADDED, "Date Added");
    validate!(headers, MEDIA_COND, "Collection Media Condition");
    validate!(headers, SLEEVE_COND, "Collection Sleeve Condition");
    validate!(headers, COLL_NOTES, "Collection Notes");
    Ok(())
}

#[inline]
fn vecify(s: &str) -> Vec<String> {
    s.split(',').map(|s| s.trim().to_string()).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_csv_header_validation() {
        let mut reader = Reader::from_path("discogs_collection.csv").unwrap();

        validate_headers(reader.headers().unwrap()).unwrap();
    }
}