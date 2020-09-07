use cursive::{
    Cursive,
    views::{
        NamedView, 
        LinearLayout,
        Panel,
        ResizedView,
        SelectView,
        ScrollView,
    },
    view::SizeConstraint,
    traits::*
};

use crate::app::{
    request::{self, ParseType},
    Release,
};
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
                            popup::albuminfo(item)
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
