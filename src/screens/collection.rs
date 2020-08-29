use cursive::views::*;
use cursive::traits::*;
use cursive::Cursive;
use cursive::view::SizeConstraint;

use crate::app::request;

use crate::app::message::{Message, MessageKind};

#[derive(Debug, Clone, Copy)]
pub struct Collection {}

#[allow(dead_code)]
impl Collection {
    pub fn new() -> Self {
        Collection{}
    }
    pub fn build(&self) -> NamedView<LinearLayout> {
        let collection = LinearLayout::horizontal()
            .child(Panel::new(ResizedView::new(
                SizeConstraint::Fixed(35), 
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<String>::new()
                        .with_name("folderlist")))))
            .child(Panel::new(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<String>::new()
                    .with_all_str(load_data())
                    .with_name("albumlist")))))
            .with_name("main_view");
                    
        collection
    }
}

pub fn add_to_list(s: &mut Cursive, name: &str, to_add: &str) {
    s.call_on_name(name, |view: &mut SelectView<String>| {
        view.add_item_str(to_add);
    });
}

pub fn format_columns (list: Vec<request::Release>) -> Vec<String> {
    //formats a vector of Release structs into an iterator of formatted strings
    //might move this to a dedicated utils module if enough helper funcs are added
    
    Vec::new() //returning an empty vector just so i can compile
}

fn load_data() -> Vec<String> {
    let data = request::query("discogs_collection.json");
    let mut titlelist = Vec::<String>::new();
    for release in data {
        titlelist.push(release.title);
    }
    titlelist
}