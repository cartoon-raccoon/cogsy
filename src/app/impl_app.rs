use cursive::Cursive;
use cursive::views::*;

#[allow(unused_imports)]
use crate::commands::{Command, CommandError};
use crate::app::{App, Release, Folders};
use crate::app::message::{Message, MessageKind};
use crate::collection::Collection;
use crate::app::request::*;

impl App {
    pub fn initialize() -> Self {
        App {
            user_id: String::from("hello"),
            token: String::from("welcome to cogsy"),
            message: Message {
                msg: String::from("Cogsy v0.1.0"),
                kind: MessageKind::Info
            },
            collection: Collection::new(),
        }
    }

    #[allow(unused_assignments)]
    pub fn execute(&mut self, s: &mut Cursive, result: Result<Command, CommandError>) {
        let mut view_content = String::new();
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
                        let updateres = fullupdate(ParseType::Collection, "discogs_collection.json");
                        match updateres {
                            Ok(releases) => {
                                self.collection.contents = releases;
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
                        //update user id in db
                    }
                    Command::UpdateToken(tk) => {
                        view_content = format!("Your token has been set to: {}", tk);
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

//? Place this in a separate file?
impl Folders {
    pub fn get(name: String) -> Vec<Release> {
        Vec::new() //returning empty vector to compile
    }
}
