use std::collections::BTreeMap;

use cursive::{
    views::*,
    view::SizeConstraint,
};
use chrono::{
    Duration,
    Date,
    DateTime,
    Utc,
};
use crate::utils::{self, Config};
use crate::app::{
    ListenLog,
    database::query,
};

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
    pub fn build_history(&self) -> Panel<
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
    pub fn build_sparkview(&self) -> Panel<
        ResizedView<
        ScrollView<
        SelectView<String>>>> {
            let list: Vec<String> = self.build_sparkview_btreemap()
                .into_iter().map(|(mut k, v)| {
                    k.truncate(40);
                    format!("{:40} | {}", k, v)
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

    //TODO: Optimise this
    pub fn build_sparkview_btreemap(&self) -> BTreeMap<String, String> {
        //* compressing ListenLog into date | listens on that day -> listengraph
        let mut listengraph = BTreeMap::<Date<Utc>, Vec<String>>::new();
        for (datetime, string) in &self.contents {
            let date = datetime.date();
            if listengraph.contains_key(&date) {
                if let Some(x) = listengraph.get_mut(&date) {
                    x.push(string.clone());
                }
            } else {
                let mut titlevec: Vec<String> = Vec::new();
                titlevec.push(string.clone());
                listengraph.insert(date, titlevec);
            }
        };

        //* padding out listengraph with unused dates
        let today = utils::get_utc_now().date();
        let mut first_date = listengraph.iter().next().unwrap().0.clone();
        while first_date < today {
            if !listengraph.contains_key(&first_date) {
                listengraph.insert(first_date.clone(), Vec::new());
            }
            first_date = first_date.checked_add_signed(Duration::days(1))
                .unwrap_or(first_date); //will result in infinite loop if overflow
        }

        //* drawing the sparkview
        let mut final_graph = BTreeMap::<String, String>::new();
        for title in query::all_titles()
        .unwrap_or_else(|_s| panic!("Error while reading from database")) {
            let mut listen_sparkview = String::new();
            for (_date, vec) in &listengraph {
                //listens for that album on that particular day
                let listens = vec.iter().filter(|title2| title == **title2).count();
                listen_sparkview.push(match listens {
                    0 => ' ',
                    1 => '▁',
                    2 => '▃',
                    3 => '▅',
                    _ => '▇'
                });
            }
            final_graph.insert(title, listen_sparkview);
        }
        final_graph
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