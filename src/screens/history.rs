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
use crate::utils;
use crate::CONFIG;
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
            Err(e) => panic!("{}", e.to_string())
        }
    }
    pub fn build_history_title(&self) -> Panel<
            ResizedView<
            ScrollView<
            SelectView<String>>>> {
        let list: Vec<String> = self.contents.iter().map(|(k, _v)| {
            let nk = k.with_timezone(&CONFIG.timezone());
            format!("{}", nk.format("%a %d %b %Y, %l:%M%P"))
        }).rev().collect();
        self.build_panel(list)
    }
    pub fn build_history(&self) -> Panel<
            ResizedView<
            ScrollView<
            SelectView<String>>>> {
        let list: Vec<String> = self.contents.iter().map(|(k, v)| {
            let nk = k.with_timezone(&CONFIG.timezone());
            format!("{} | {}", nk.format("%a %d %b %Y, %l:%M%P"), v)
        }).rev().collect();
        self.build_panel(list)
    }

    fn build_panel(&self, list: Vec<String>) -> Panel<
            ResizedView<
            ScrollView<
            SelectView<String>>>> {
        Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<String>::new()
                .with_all_str(list)
            )
        ))
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
            Panel::new(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<String>::new()
                    .with_all_str(list)
                )
            ))
        }

    //TODO: Optimise this
    //? What is its Big O / space-time complexity?
    #[allow(clippy::map_entry)]
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
                listengraph.insert(date, vec![string.clone()]);
            }
        };

        //* padding out listengraph with unused dates
        let today = utils::get_utc_now().date();
        let first_date = *listengraph.iter().next()
            .unwrap_or((&today, &vec![String::new()]))
            .0;
        let earliest_usable_date = today.checked_sub_signed(Duration::days(80))
            .unwrap_or(today);
        let mut date_to_use = if first_date > earliest_usable_date 
            {first_date} else {earliest_usable_date};
        while date_to_use < today {
            if !listengraph.contains_key(&date_to_use) {
                listengraph.insert(date_to_use, Vec::new());
            }
            date_to_use = date_to_use.checked_add_signed(Duration::days(1))
                .unwrap_or(date_to_use); //will result in infinite loop if overflow
        }

        //* drawing the sparkview
        let mut final_graph = BTreeMap::<String, String>::new();
        for title in query::all_titles().unwrap_or_else(|_| vec![]) {
            let mut listen_sparkview = String::new();
            for vec in listengraph.values() {
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

        self.contents.insert(time, title)
    }
}