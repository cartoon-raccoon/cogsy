use std::collections::BTreeMap;

use cursive::views::*;
use chrono::{
    DateTime,
    Utc,
};
use crate::app::{
    ListenLog,
    database::query,
};

#[allow(dead_code)]
impl ListenLog { //wrapper around a BTreeMap
    pub fn new() -> Self {
        let new_self: BTreeMap<DateTime<Utc>, String> = BTreeMap::new();
        ListenLog {
            contents: new_self,
        }
    }
    pub fn init() -> Self {
        match query::listenlog() {
            Ok(listenlog) => listenlog,
            Err(e) => panic!(e.to_string())
        }
    }
    pub fn build(&self) -> StackView {
        let screen = StackView::new();
        screen
    }
    pub fn contents(&mut self) -> BTreeMap<DateTime<Utc>, String> {
        self.contents.clone()
    }
    pub fn push(&mut self,
        time: DateTime<Utc>,
        title: String) -> Option<String> {

        match self.contents.insert(time, title) {
            None => None,
            Some(old_val) => Some(old_val),
        }
    }
}