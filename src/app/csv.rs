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

macro_rules! ok_or {
    ($e:expr, $idx:expr) => {
        $e.get($idx).ok_or(UpdateError::CSVParseError)
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

impl From<std::num::ParseIntError> for UpdateError {
    fn from(_: std::num::ParseIntError) -> UpdateError {
        UpdateError::CSVParseError
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

impl Release {
    pub fn from_stringrecord(record: &StringRecord) -> Result<Self, UpdateError> {
        Ok(Self {
            id: ok_or!(record, RELEASE_ID)?.parse()?,
            search_string: unidecode(ok_or!(record, TITLE)?)
            .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], ""),
            title: ok_or!(record, TITLE)?.to_string(),
            artist: ok_or!(record, ARTIST)?.to_string(),
            year: ok_or!(record, RELEASED)?.parse()?,
            labels: vecify(ok_or!(record, LABEL)?),
            formats: vecify(ok_or!(record, FORMAT)?),
            date_added: {
                let added_date = DateTime::parse_from_rfc3339(ok_or!(record, DATE_ADDED)?)
                .unwrap_or_else(|_| utils::get_utc_now()
                .with_timezone(&CONFIG.timezone()));

                DateTime::<Utc>::from_utc(
                    added_date.naive_utc(), Utc
                )
            }
        })
    }
}

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Release>, UpdateError> {
    let mut reader = Reader::from_path(path)?;

    validate_headers(reader.headers()?)?;

    let mut ret = Vec::new();
    for record in reader.records() {
        ret.push(Release::from_stringrecord(&record?)?)
    }

    Ok(ret)
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