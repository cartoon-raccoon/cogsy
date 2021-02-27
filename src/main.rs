mod app;
mod screens;
mod utils;
mod commands;

use std::process::exit;

use cursive::{
    Cursive,
    traits::*,
    views::*,
    event::{Event, Key},
};
use app::fromcli;
use app::App;
use commands::{Command};
use screens::{
    collection, 
};

fn main() {
    let clapapp = fromcli::init().get_matches();
    if let Some(_) = clapapp.subcommand_matches("reset") {
        exit(fromcli::handle_reset(utils::Config::load()).unwrap());
    }
    let mut app = App::initialize();
    
    if let Some(status) = fromcli::parse_and_execute(clapapp, &app) {
        exit(status);
    }
    
    let mut siv = cursive::default();
    siv.set_theme(utils::theme_gen(app.colour));

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
            s.clear_global_callbacks(Event::Key(Key::Esc));
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

    App::add_callbacks(&mut siv);

    siv.run();
}