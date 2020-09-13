use std::collections::BTreeMap;

use cursive::{
    views::*,
    view::SizeConstraint,
};
use chrono::{
    DateTime,
    Utc,
};
use crate::utils::Config;
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
    pub fn build(&self) -> Panel<
            ResizedView<
            ScrollView<
            SelectView<String>>>> {
        let list: Vec<String> = self.contents.iter().map(|(k, v)| {
            let nk = k.with_timezone(&Config::timezone());
            format!("{} | {}", nk.format("%a %d %b %Y, %l:%M%P"), v)
        }).collect();
        let screen = Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<String>::new()
                .with_all_str(list)
            )
        ));
        screen
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