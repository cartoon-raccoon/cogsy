mod app;
mod screens;
mod theme;
mod commands;

use std::process::exit;
use cursive::{
    Cursive,
    traits::*,
    views::*,
    event::{Event, Key}
};
use app::{
    App,
    database::*,
};
use commands::{Command};
use screens::{
    collection, 
    wantlist::Wantlist
};

fn main() {
    let mut app = App::initialize();

    if !admin::check_integrity() {
        println!("Database integrity check failed, exiting.");
        exit(1);
    } //TODO: add db-config username matching
    
    let mut siv = cursive::default();
    siv.set_theme(theme::theme_gen());

    //initialize screen data
    let collectscreen = app.collection.build();

    //building gui tree
    let message = TextContent::new(app.message.msg.clone());
    let messagebox = TextView::new_with_content(message.clone())
        .with_name("messagebox");
    
    let commandline = EditView::new()
        .on_submit_mut( move |s: &mut Cursive, text| {
            s.focus_name("albumlist").unwrap();
            s.call_on_name("commandline", |view: &mut EditView| {
                view.set_content("");
                view.disable();
            });
            let result = Command::parse(text);
            app.execute(s, result);
        })
        .disabled()
        .with_name("commandline");

    let main_layout = LinearLayout::vertical()
        .child(collectscreen)
        .child(messagebox)
        .child(commandline);
    
    siv.add_fullscreen_layer(main_layout);

    //adding callbacks
    siv.add_global_callback('q', |s| {
        //TODO: check app modified state and write to file
        s.quit();
    });
    siv.add_global_callback(':', |s| {
        s.call_on_name("commandline", |view: &mut EditView| {
            view.enable();
            view.set_content(":");
        });
        s.focus_name("commandline").unwrap();
    });
    //TODO: implement commands to handle opening of child screens
    siv.add_global_callback(Event::Key(Key::Backspace), |s| {
        if s.screen().len() > 1 {
            s.pop_layer();
        }
    });
    siv.add_global_callback('1', move |s| {
        if s.screen().len() == 1 {
            s.add_fullscreen_layer(Wantlist::init().build());
        }
    });
    //placeholder code until folders are implemented
    collection::add_to_list(&mut siv, "folderlist", "All");
    collection::add_to_list(&mut siv, "folderlist", "Uncategorized");

    siv.run();
}