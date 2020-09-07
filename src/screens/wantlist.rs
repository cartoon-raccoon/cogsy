use cursive::{
    views::*,
    view::SizeConstraint,
    traits::*
};
use crate::app::{
    {Release},
    request::{self, ParseType},
};
use crate::screens::popup;

//use crate::app::message::{Message, MessageKind};

#[derive(Debug, Clone)]
pub struct Wantlist {
    wantlist: Vec<Release>
}

impl Wantlist {
    pub fn init() -> Self {
        Wantlist{ //placeholder: will read from database
            wantlist: request::parse_releases(
                ParseType::Wantlist(String::from("hi")), 
                "discogs_wantlist.json", true).unwrap()
        }
    }
    pub fn build(&self) -> Panel< //long-ass return type declaration
            ResizedView<
            ScrollView<
            NamedView<
            SelectView<Release>>>>> {
        //iterator members must be formatted into columns
        let main_screen = Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<Release>::new()
                .with_all(self.wantlist.clone().into_iter().map(|i| {
                    (i.title.clone(), i)
                }))
                .on_submit(|s, item| {
                    s.add_fullscreen_layer(
                        popup::albuminfo(item)
                    );
                })
                .with_name("wantlist"))
        ));
        main_screen
    }
}

//TODO: Implement formatting of titles
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
