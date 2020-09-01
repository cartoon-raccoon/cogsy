use cursive::Cursive;
use cursive::views::*;

#[allow(unused_imports)]
use crate::commands::{Command, CommandError};
use crate::app::App;
use crate::app::message::{Message, MessageKind};

impl App {
    pub fn initialize() -> Self {
        App {
            user_id: String::from("hello"),
            token: String::from("welcome to cogsy"),
            message: Message {
                msg: String::from("Welcome to Cogsy"),
                kind: MessageKind::Info
            }
        }
    }

    //TODO: Implement this shit (PRIORITY)
    pub fn execute(&self, s: &mut Cursive, result: Result<Command, CommandError>) {
        match result {
            Ok(_r) => {
                s.call_on_name("messagebox", |view: &mut TextView| {
                    view.set_content("command successfully parsed!");
                });
            }
            Err(_e) => {
                s.call_on_name("messagebox", |view: &mut TextView| {
                    view.set_content("there was an error parsing the command");
                });
            }
        }
    }

}
