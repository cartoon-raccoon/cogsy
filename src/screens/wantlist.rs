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
    pub fn build(&self) -> Panel<
            ResizedView<
            ScrollView<
            NamedView<
            SelectView>>>> {
        let main_screen = Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<String>::new()
                .item_str("hello this is the wantlist")
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