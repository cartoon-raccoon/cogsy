use std::fmt;

use crate::app::App;
use regex::Regex;

/*
The architecture of this code (and a lot of the code itself) was shamelessly stolen from:
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

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
pub enum Command {
    UpdateDB,               //switch and argument
    UpdateID(String),       //username
    UpdateToken(String),    //token
    Random(bool),           //true = nolog
    Price(String, f64),     //album name, price
    Listen(String, String), //album name
    Query(String),          //album name
    Empty,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CommandError {
    InvalidCommand(String),
    InvalidArg(u32),
    InvalidAlbum(String),
    InvalidSyntax(String, String),
    NotEnoughArgs(String, u32),
    TooManyArgs(String, u32),
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::InvalidCommand(s) => write!(f, "Invalid command: `{}`", s),
            CommandError::InvalidArg(p) => write!(f, "Invalid argument at position {}", p),
            CommandError::InvalidAlbum(n) => write!(f, "Invalid album name given: `{}`", n),
            CommandError::InvalidSyntax(n, s) => {
                write!(f, "Invalid syntax for command `{}`: Could not parse `{}`", n, s)
            },
            CommandError::NotEnoughArgs(s, n) => {
                write!(f, "Error: Command `{}` requires at least {} argument(s).", s, n)
            },
            CommandError::TooManyArgs(s, n) => {
                write!(f, "Error Command `{}` only requires at most {} argument(s).", s, n)
            },
        }
    }
}

/* 
*Because album names can have more than one word, the command text cannot be split by whitespace
*Instead, regexes have to be used to match a string of text enclosed in double quotes
*Then the text following the matched pattern can be interpreted normally
*/
impl Command {
    pub fn parse(input: &str) -> Result<Command, CommandError> {

        //closure to capture regexes and returns a Vec of command arguments
        let splitter = |cmdtext: &str| -> Vec<String> {
            let re = Regex::new("\".*\"").unwrap();
            let mut args: Vec<String> = re.split(cmdtext).map(|arg| arg.to_string()).collect();
            let target = re.captures(cmdtext);
            match target {
                Some(target) => {
                    let target = target.get(0).unwrap().as_str();
                    args.push(target.to_string());
                    for arg in &mut args {
                        arg.retain(|c| c != '"');
                    }
                    args
                },
                None => {
                    for arg in &mut args {
                        arg.retain(|c| c != '"');
                    }
                    args
                }
            }
        };

        let strings: Vec<&str> = input.trim().split(' ').collect();
        if strings.is_empty() {
            return Ok(Command::Empty);
        }
        let first = strings.first().unwrap().to_string();
        match first.as_str() {
            ":update" => {
                if strings.len() > 3 {
                    return Err(CommandError::TooManyArgs(first, 2))
                } else if strings.len() == 1 {
                    return Ok(Command::UpdateDB);
                } else {
                    match strings[1] {
                        "-u" | "--username" => {
                            return Ok(Command::UpdateID(strings[2].to_string()));
                        },
                        "-t" | "--token" => {
                            return Ok(Command::UpdateToken(strings[2].to_string()));
                        },
                        _ => {
                            return Err(CommandError::InvalidSyntax(
                                first, 
                                strings[1].to_string()));
                        }
                    }
                }
            },
            ":random" => {
                if strings.len() > 2 {
                    return Err(CommandError::TooManyArgs(first, 1))
                } else if strings.len() == 2 {
                    match strings[1] {
                        "-n" | "--nolog" => {
                            return Ok(Command::Random(true));
                        }
                        _ => {
                            return Err(CommandError::InvalidSyntax(
                                first,
                                strings[1].to_string()));
                        }
                    } 
                } else {
                    return Ok(Command::Random(false));
                }
            },
            ":price" => {
                let mut argv = splitter(input.trim());
                if argv.len() < 2 {
                    return Err(CommandError::NotEnoughArgs(first, 2));
                }
                let extra_args: Vec<&str> = argv[1].trim().split(' ').collect();
                if extra_args.len() > 1 {
                    return Err(CommandError::TooManyArgs(first, 2))
                } else if argv[1] == "" {
                    return Err(CommandError::NotEnoughArgs(first, 2))
                } else {
                    argv[1].retain(|c| c != ' ');
                    //* This panics if not passed numbers
                    let price = argv[1].parse::<f64>();
                    match price {
                        Ok(price) => {
                            return Ok(Command::Price(argv[2].clone(), price));},
                        Err(_e) => {
                            return Err(CommandError::InvalidSyntax(
                                argv[0].clone(), 
                                argv[1].clone()));
                            }
                    }
                }
            },
            ":listen" => {
                let mut argv = splitter(input.trim());
                if argv.len() < 2 {
                    return Err(CommandError::NotEnoughArgs(first, 2));
                }
                let extra_args: Vec<&str> = argv[1].trim().split(' ').collect();
                if extra_args.len() > 1 {
                    return Err(CommandError::TooManyArgs(first, 2));
                } else {
                    argv[1].retain(|c| c != ' ');
                    return Ok(Command::Listen(argv[2].clone(), argv[1].clone()));
                }
            },
            ":query" => {
                if strings.len() == 1 {
                    return Err(CommandError::NotEnoughArgs(first, 1));
                }
                let argv = splitter(input.trim());
                if argv[1] != "" {
                    return Err(CommandError::TooManyArgs(first, 1));
                } else if argv.len() < 3 {
                    return Err(CommandError::NotEnoughArgs(first, 1));
                } else {
                    return Ok(Command::Query(argv[2].clone()));
                }
            },
            _ => {
                return Err(CommandError::InvalidCommand(first));
            },
        }
    }
}