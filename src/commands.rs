use std::fmt;

use crate::app::App;

/*
The architecture this code (and a lot of the code itself) was shamelessly stolen from:
github.com/NerdyPepper/dijo/blob/master/src/command.rs and modified to fit my app.
Just giving credit where it's due.
*/

#[allow(dead_code)]
static COMMANDS: &'static [&'static str] = &[
    "update",
    "random",
    "price",
    "listen",
    "query",
];

fn get_command_completion(prefix: &str) -> Option<String> {
    let first_match = COMMANDS.iter().filter(|&x| x.starts_with(prefix)).next();
    return first_match.map(|&x| x.into());
}

#[derive(PartialEq)]
pub enum Command {
    Update(String, String), //switch and argument
    Random(String),         //album name
    Price(String, f64),     //album name, price
    Listen(String),         //album name
    Query(String),          //album name
}

#[derive(Debug)]
pub enum CommandError {
    InvalidCommand(String),
    InvalidArg(u32),
    NotEnoughArgs(String, u32),
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::InvalidCommand(s) => write!(f, "Invalid command: `{}`", s),
            CommandError::InvalidArg(p) => write!(f, "Invalid argument at position {}", p),
            CommandError::NotEnoughArgs(s, n) => {
                write!(f, "Command `{}` requires at least {} argument(s)!", s, n)
            }
        }
    }
}

impl Command {
    pub fn from_string(&self) {

    }
}