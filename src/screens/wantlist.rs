use cursive::Cursive;
use cursive::views::*;
use cursive::view::SizeConstraint;
use cursive::traits::*;

use crate::app::message::{Message, MessageKind};

#[derive(Debug, Clone)]
pub struct Wantlist {
    wantlist: Vec<String>
}

impl Wantlist {
    pub fn new() -> Self {
        Wantlist{
            wantlist: vec![
                String::from("hello"), 
                String::from("this"),
                String::from("is"), 
                String::from("cogsy")
            ]
        }
    }
    pub fn build(&self) -> Panel< //long-ass return type declaration
            ResizedView<
            ScrollView<
            NamedView<
            SelectView>>>> {
        //placeholder vector (what's new)
        //iterator members must be formatted into columns
        let main_screen = Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            ScrollView::new(
                SelectView::<String>::new()
                //add an iterator to add all the entries
                .with_all_str(&self.wantlist)
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