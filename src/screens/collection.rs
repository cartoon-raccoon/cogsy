use cursive::views::*;
use cursive::traits::*;
use cursive::Cursive;
use cursive::view::SizeConstraint;

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