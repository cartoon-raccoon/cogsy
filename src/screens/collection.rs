use cursive::views::*;
use cursive::traits::*;
use cursive::Cursive;
use cursive::view::SizeConstraint;

use crate::app::request::{self, ParseType};
use crate::app::Release;
use crate::screens::popup;
//use crate::app::message::{Message, MessageKind};

#[derive(Debug, Clone)]
pub struct Collection {
    pub contents: Vec<Release>
}

impl Collection {
    pub fn new() -> Self {
        Collection{
            //this will eventually call database::query::get_from_db()
            contents: request::parse_releases(
                ParseType::Collection,
                "discogs_collection.json",
                true).unwrap()
        }
    }
    pub fn build(&self) -> NamedView<LinearLayout> {
        let collection = LinearLayout::horizontal()
            .child(Panel::new(ResizedView::new(
                SizeConstraint::Fixed(35), 
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<String>::new()
                        // .on_submit(|s, text| {

                        // })
                        .with_name("folderlist")))))
            .child(Panel::new(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<Release>::new()
                    .with_all(self.contents.clone().into_iter().map(|i| {
                        (i.title.clone(), i)
                    }))
                    .on_submit(|s, item| {
                        s.add_fullscreen_layer(
                            popup::build(item.clone())
                        );
                    })
                    .with_name("albumlist")))))
            .with_name("main_view");
                    
        collection
    }

    pub fn refresh(&mut self, s: &mut Cursive) {
        //update from database and reload its contents
        //call database method here
        s.call_on_name("albumlist", |view: &mut SelectView<Release>| {
            view.clear();
            view.add_all(self.contents.clone().into_iter().map(|i| {
                (i.title.clone(), i)
            }))
        });
    }
}

pub fn add_to_list(s: &mut Cursive, name: &str, to_add: &str) {
    s.call_on_name(name, |view: &mut SelectView<String>| {
        view.add_item_str(to_add);
    });
}

pub fn format_columns (list: Vec<Release>) -> Vec<String> {
    //formats a vector of Release structs into an iterator of formatted strings
    //might move this to a dedicated utils module if enough helper funcs are added

    /*
    Step 1: Iterate over the vec to find the entry with the longest title
    Step 2: Get its length and append a buffer of 5 spaces long (global)
            (This will be the position at which to place the artist name)
    Step 3: For each title in vec:
            Find the local buffer length to append:
            localbuffer = global - length
            Generate a whitespace string of that length and append it to the title
            Append the artist name
            Push the string to the new vec
    Step 4: Return!
    NOTE: This fn will consume the vector it is passed, so make sure you clone it!
    */

    Vec::new() //returning an empty vector just so i can compile
}
