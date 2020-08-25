use cursive::Cursive;
use cursive::views::*;
use cursive::view::SizeConstraint;
use cursive::traits::*;

use crate::app::message::{Message, MessageKind};

pub struct Wantlist {

}

impl Wantlist {
    pub fn new() -> Self {
        Wantlist{}
    }
    pub fn build(&self) -> Panel< //long-ass return type declaration
            ResizedView<
            ScrollView<
            NamedView<
            SelectView>>>> {
        //placeholder vector (what's new)
        //iterator members must be formatted into columns
        let name_list = vec!["hello", "this", "is", "cogsy"];

        let main_screen = Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<String>::new()
                //add an iterator to add all the entries
                .with_all_str(name_list)
                .with_name("wantlist"))
        ));
        
        main_screen
    }
}

pub fn add_to_list(s: &mut Cursive, name: &str, to_add: &str) {
    s.call_on_name(name, |view: &mut SelectView<String>| {
        view.add_item_str(to_add);
    });
}