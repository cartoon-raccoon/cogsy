use std::collections::HashMap;
use std::fs::read_to_string;

use cursive::{
    Cursive,
    views::*,
};
use crate::app::{
    {App, Release, Folders},
    database::*,
    message::{Message, MessageKind},
    update,
};
use crate::collection::Collection;
use crate::commands::{Command, CommandError};

impl App {
    pub fn initialize() -> Self {
        App {
            user_id: String::from("cartoon.raccoon"),
            token: read_to_string("discogs_token").unwrap(),
            message: Message {
                msg: String::from("Cogsy v0.1.0"),
                kind: MessageKind::Info
            },
            collection: Collection::new(),
            modified: false
        }
    }

    pub fn execute(&mut self, s: &mut Cursive, result: Result<Command, CommandError>) {
        let view_content: String;
        match result {
            Ok(command) => {
                match command {
                    //*for every command that updates some data, the app must:
                    //*1: Update the internal state in memory
                    //*2: Update the database
                    Command::UpdateDB => {
                        s.call_on_name("messagebox", |view: &mut TextView| {
                            view.set_content("Updating collection...");
                        });
                        let updateres = update::full(self.user_id.clone(), self.token.clone(), false);
                        match updateres {
                            Ok(()) => {
                                //*Placeholder code (again)
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
                        if nolog {
                            view_content = "You should play: (No logging)".to_string();
                        } else {
                            view_content = "You should play: ".to_string();
                        }
                    }
                    Command::Price(album, _price) => {
                        view_content = format!("Setting the price of `{}`", album);
                    }
                    Command::Listen(album, _time) => {
                        view_content = format!("Listening to: `{}`", album);
                    }
                    Command::Query(album) => {
                        view_content = format!("Querying database for: `{}`", album);
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
}

#[allow(dead_code)]
impl Folders { //wrapper around a HashMap<String, Vec<Release>>
    pub fn new() -> Self {
        let new_self: 
            HashMap<String, Vec<Release>> = HashMap::new();
        Folders {
            contents: new_self,
        }
    }
    pub fn contents(&mut self) -> HashMap<String, Vec<Release>> {
        self.contents.clone()
    }

    pub fn pull(&mut self,
        name: &str) -> Option<Vec<Release>> {
        
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
