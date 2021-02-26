use std::fs;
use std::io;
use std::path::Path;
use std::process::exit;

use std::collections::BTreeMap;
use cursive::{
    Cursive,
    views::*,
    event::{Event, Key},
};
use crate::app::{
    {
        App, 
        Release, 
        Folders, 
        ListenLogEntry,
        ListenLog,
    },
    database::{
        admin, 
        query, 
        update as dbupdate, 
        query::QueryType
    },
    message::{Message, MessageKind},
    update,
};
use crate::utils::{self, Config};
use crate::screens::{
    Wantlist,
    popup,
    profile,
};
use crate::collection::Collection;
use crate::commands::{Command, CommandError};

const DB_NOT_INIT_MSG: &str =
"Database not initialized. Would you like to initialize it now? [Y/n]";

const DB_INTEGRITY_FAIL_MSG: &str =
"Database integrity check failed, would you like to re-initialize it now? [Y/n]";

fn on_init_fail(username: &str, token: &str) {
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)
        .expect("Oops, could not read line.");
    match answer.as_str() {
        "Y\n" | "y\n" | "yes\n" | "Yes\n" => {
            println!("Beginning database initialization.");
            match update::full(username, token, true, false) {
                Ok(()) => {}
                Err(e) => {
                    println!("\n{}", e);
                    fs::remove_file(utils::database_file()).unwrap();
                    exit(1);
                }
            }
        },
        "N\n" | "n\n" | "no\n" | "No\n" => {exit(1);},
        _ => {println!("{}", Message::set("Please choose Y/N", MessageKind::Error)); exit(1);}
    }
}

impl App {
    pub fn initialize() -> Self {
        if !Path::new(&utils::config_file()).exists() {
            Config::first_init();
        }
        let config = Config::load();
        let token = format!(
            "Discogs token={}",
            config.token
        );
        let dbfilepath = utils::database_file();

        if !Path::new(&dbfilepath).exists() {
            println!("{}", Message::set(DB_NOT_INIT_MSG, MessageKind::Hint));
            on_init_fail(&config.username, &token);
        }
        if !utils::usernames_match() {
            println!("{}", 
                Message::set(
                    "The username in your config file seems to have changed.", 
                    MessageKind::Hint
                )
            );
            println!("Would you like to use the new username? [Y/n]");
            on_init_fail(&config.username, &token);
        }
        if !admin::check_integrity() {
            println!("{}", Message::set(DB_INTEGRITY_FAIL_MSG, MessageKind::Hint));
            on_init_fail(&config.username, &token);
        }

        App {
            user_id: config.username,
            token: token,
            message: Message {
                msg: String::from(format!("Cogsy v{}", env!("CARGO_PKG_VERSION"))),
                kind: MessageKind::Info
            },
            collection: Collection::new(),
            modified: false
        }
    }

    pub fn execute(&mut self, s: &mut Cursive, result: Result<Command, CommandError>) {
        let mut view_content: String;
        match result {
            Ok(command) => {
                match command {
                    Command::UpdateDB => {
                        s.call_on_name("messagebox", |view: &mut TextView| {
                            view.set_content("Updating collection...");
                        });
                        let updateres = update::full(&self.user_id, 
                                                     &self.token, 
                                                     false, false);
                        match updateres {
                            Ok(()) => {
                                self.collection.folders = query::collection().unwrap();
                                self.collection.refresh(s);
                                view_content = "Database successfully updated.".to_string();
                            }
                            Err(e) => {
                                view_content = e.to_string();
                            }
                        }
                    }
                    Command::UpdateID(id) => {
                        view_content = format!(
                            "Your id has been set to `{}`, restart the app for the changes.",
                            id);
                        self.user_id = id;
                        self.modified = true;
                    }
                    Command::UpdateToken(tk) => {
                        view_content = format!("Your token has been set to: {}", tk);
                        self.token = format!("Discogs token={}", tk);
                        self.modified = true;
                    }
                    Command::Random(nolog) => {
                        match query::random() {
                            Ok(random) => {
                                if !nolog {
                                    let time_now = utils::get_utc_now();
                                    let entry = ListenLogEntry {
                                        id: random.id,
                                        title: random.title.clone(),
                                        time: time_now,
                                    };
                                    match dbupdate::listenlog(entry) {
                                        Ok(()) => {
                                            view_content = format!("You should play `{}`", random.title);
                                        }
                                        Err(e) => {view_content = e.to_string();}
                                    }
                                } else {
                                    view_content = format!("You should play `{}`", random.title);
                                }
                            },
                            Err(e) => {
                                view_content = e.to_string();
                            }
                        };
                    }
                    Command::Price(_album, _price) => {
                        view_content = format!("Sorry, the price command is not supported at this time.");
                    }
                    Command::Listen(album, _time) => {
                        match query::release(album.clone(), QueryType::Collection) {
                            Ok(results) => {
                                if results.len() > 1 {
                                    view_content = format!("Multiple results for {}", album);
                                    s.add_fullscreen_layer(
                                        //listenlog gets logged internally here
                                        popup::multiple_results(results, true)
                                    );
                                } else if results.len() == 1 {
                                    let time_now = utils::get_utc_now();
                                    let entry = ListenLogEntry {
                                        id: results[0].id,
                                        title: results[0].title.clone(),
                                        time: time_now,
                                    };
                                    match dbupdate::listenlog(entry) {
                                        Ok(()) => {view_content = format!("Listening to `{}`", results[0].title);}
                                        Err(e) => {view_content = e.to_string();}
                                    }
                                } else {
                                    view_content = format!("Unable to find results for `{}`", album);
                                }

                            }
                            Err(e) => {view_content = format!("{}", e);}
                        }
                    }
                    Command::Query(album) => {
                        match query::release(album.clone(), QueryType::Collection) {
                            Ok(results) => {
                                view_content = format!("Querying collection for `{}`", album);
                                if results.len() > 1 {
                                    s.add_fullscreen_layer(
                                        popup::multiple_results(results, false)
                                    );
                                } else if results.len() == 1 {
                                    s.add_fullscreen_layer(
                                        popup::albuminfo(results[0].clone())
                                    );
                                } else {
                                    view_content = format!("Unable to find results for `{}`", album);
                                }
                            }
                            Err(e) => {view_content = format!("{}", e);}
                        }
                    }
                    Command::QueryWantlist(album) => {
                        match query::release(album.clone(), QueryType::Wantlist) {
                            Ok(results) => {
                                view_content = format!("Querying wantlist for `{}`", album);
                                if results.len() > 1 {
                                    s.add_fullscreen_layer(
                                        popup::multiple_results(results, false)
                                    );
                                } else if results.len() == 1 {
                                    s.add_fullscreen_layer(
                                        popup::albuminfo(results[0].clone())
                                    );
                                } else {
                                    view_content = format!("Unable to find results for `{}`", album);
                                }
                            }
                            Err(e) => {view_content = format!("{}", e);}
                        }
                    }
                    Command::Empty => {
                        view_content = "Empty command".to_string();
                    }
                }
                s.call_on_name("messagebox", |view: &mut TextView| {
                    view.set_content(view_content);
                });
            }
            Err(error) => {
                s.call_on_name("messagebox", |view: &mut TextView| {
                    view.set_content(error.to_string());
                });
            }
        }
    }

    pub fn add_callbacks(s: &mut Cursive) {
        s.add_global_callback('q', |s| {
            //TODO: check app modified state and write to file
            s.quit();
        });
        s.add_global_callback(':', |s| {
            s.call_on_name("commandline", |view: &mut EditView| {
                view.enable();
                view.set_content(":");
            });
            s.add_global_callback(Event::Key(Key::Esc), |s| {
                s.focus_name("albumlist").unwrap();
                s.call_on_name("commandline", |view: &mut EditView| {
                    view.set_content("");
                    view.disable();
                });
                s.clear_global_callbacks(Event::Key(Key::Esc));
            });
            s.focus_name("commandline").unwrap();
        });
        s.add_global_callback(Event::Key(Key::Backspace), |s| {
            if s.screen().len() > 1 {
                s.pop_layer();
            }
        });
        //collection screen
        s.add_global_callback('1', |s| {
            while s.screen().len() > 1 {
                s.pop_layer();
            }
        });
        //wantlist screen
        s.add_global_callback('2', |s| {
            while s.screen().len() > 1 {
                s.pop_layer();
            }
            if s.screen().len() == 1 {
                s.add_fullscreen_layer(Wantlist::init().build());
            }
        });
        //profile screen
        s.add_global_callback('3', |s| {
            while s.screen().len() > 1 {
                s.pop_layer();
            }
            if s.screen().len() == 1 {
                s.add_fullscreen_layer(profile::build());
            }
        });
        s.add_global_callback('4', |s| {
            while s.screen().len() > 1 {
                s.pop_layer();
            }
            if s.screen().len() == 1 {
                s.add_fullscreen_layer(ListenLog::init().build_sparkview());
            }
        });
        s.add_global_callback('h', |s| {
            while s.screen().len() > 1 {
                s.pop_layer();
            }
            if s.screen().len() == 1 {
                s.add_fullscreen_layer(ListenLog::init().build_history());
            }
        });
    }
}

#[allow(dead_code)]
impl Folders { //wrapper around a BTreeMap<String, Vec<Release>>
    pub fn new() -> Self {
        let new_self: 
            BTreeMap<String, Vec<Release>> = BTreeMap::new();
        Folders {
            contents: new_self,
        }
    }
    pub fn contents(&mut self) -> BTreeMap<String, Vec<Release>> {
        self.contents.clone()
    }

    pub fn pull(&mut self, name: &str) -> Option<Vec<Release>> {
        
        match self.contents.remove(name) {
            None => None,
            Some(releases) => Some(releases),
        }
    }

    pub fn push(&mut self, 
        folder: String, 
        contents: Vec<Release>) -> Option<Vec<Release>> {
        
        match self.contents.insert(folder, contents) {
            None => None,
            Some(old_val) => Some(old_val)
        }
    }
}
