use cursive::views::*;
use cursive::traits::*;
use cursive::Cursive;
use cursive::view::SizeConstraint;

use crate::screens::traits::Screen;
use crate::app::message::Message;

#[derive(Debug, Clone, Copy)]
pub struct Collection {}

#[allow(dead_code)]
impl Screen for Collection {
    fn new() -> Self {
        Collection{}
    }
    fn build(&self) -> StackView {
        let main_view = LinearLayout::horizontal()
            .child(ResizedView::new(
                SizeConstraint::Fixed(25), 
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<String>::new()
                        .with_name("folderlist"))))
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<String>::new()
                    .with_name("albumlist"))))
            .with_name("main_view");
        
        let message = TextContent::new("Welcome to Cogsy!");
        let messagebox = TextView::new_with_content(message.clone());
        
        let commandline = EditView::new()
            .on_submit( move |s: &mut Cursive, text| {
                //placeholder code until i implement commands
                add_to_list(s, "albumlist", text);
                s.call_on_name("commandline", |view: &mut EditView| {
                    view.set_content("");
                });
                message.set_content(&format!("Added '{}'", text));
                s.focus_name("albumlist").unwrap();
            })
            .with_name("commandline");
        let layout = LinearLayout::vertical()
            .child(main_view)
            .child(messagebox)
            .child(commandline);
        let mut screen = StackView::new();
        screen.add_fullscreen_layer(layout);

        screen
    }
}

pub fn add_to_list(s: &mut Cursive, name: &str, to_add: &str) {
    s.call_on_name(name, |view: &mut SelectView<String>| {
        view.add_item_str(to_add);
    });
}