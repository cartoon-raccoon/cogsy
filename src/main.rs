mod app;
mod screens;
mod config;
mod utils;
mod commands;

#[macro_use]
extern crate lazy_static;

use std::process::exit;

use cursive::{
    Cursive,
    traits::*,
    views::*,
    event::{Event, Key},
};
use app::cli;
use app::App;
use config::{Config, Appearance};
use commands::{Command};
use screens::{
    collection, 
};

lazy_static! {
    static ref CONFIG: Config = Config::load();
    static ref APPEARANCE: Appearance = match CONFIG.appearance {
        Some(ref appearance) => {
            let mut appear = appearance.clone();
            appear.resolve();
            appear
        }
        None => {
            Appearance::default()
        }
    };
}

fn main() {
    let clapapp = cli::init().get_matches();
    if let Some(sub_m) = clapapp.subcommand_matches("database") {
        exit(cli::handle_database(sub_m).unwrap());
    }
    let mut app = App::initialize();
    
    if let Some(status) = cli::parse_and_execute(clapapp, &app) {
        exit(status);
    }
    
    let mut siv = cursive::default();
    siv.set_theme(config::theme_gen(&mut app.appearance));

    //initialize screen data
    let collectscreen = app.collection.build(
        app.appearance.folders() as usize
    );

    //building gui tree
    let message = TextContent::new(app.message.msg.clone());
    let messagebox = TextView::new_with_content(message)
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