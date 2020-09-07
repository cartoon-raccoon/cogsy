use cursive::{
    views::*,
    view::SizeConstraint,
    traits::*};

//use crate::app::message::{Message, MessageKind};

#[derive(Debug, Clone)]
pub struct Wantlist {
    wantlist: Vec<String>
}

impl Wantlist {
    pub fn init() -> Self {
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
