use std::fmt;

use regex::Regex;

/*
The architecture of this code (and a lot of the code itself) was shamelessly stolen from:
github.com/NerdyPepper/dijo/blob/master/src/command.rs and modified to fit my app.
Just giving credit where it's due.
*/

#[derive(PartialEq, Debug)]
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
pub enum CommandError {
    InvalidCommand(String),
    InvalidAlbum,
    InvalidSyntax(String, String),
    NotEnoughArgs(String, u32),
    TooManyArgs(String, u32),
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::InvalidCommand(s) => write!(f, "Invalid command: `{}`", s),
            CommandError::InvalidAlbum => {
                write!(f, "Error: Could not parse album name. Try enclosing it in double quotes.")
            }
            CommandError::InvalidSyntax(n, s) => {
                write!(f, "Invalid syntax for command `{}`: Could not parse `{}`", n, s)
            },
            CommandError::NotEnoughArgs(s, n) => {
                write!(f, "Error: Command `{}` requires at least {} argument(s).", s, n)
            },
            CommandError::TooManyArgs(s, n) => {
                write!(f, "Error: Command `{}` only requires at most {} argument(s).", s, n)
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
        let splitter = |cmdtext: &str| -> Option<Vec<String>> {
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
                    Some(args)
                },
                None => None
            }
        };

        let strings: Vec<&str> = input.trim().split(' ').collect();
        if strings.is_empty() {
            return Ok(Command::Empty);
        }
        let mut first = strings.first().unwrap().to_string();
        first.retain(|c| c != ':');
        match first.as_str() {
            "update" => {
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
            "random" => {
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
            "price" => {
                if strings.len() == 1 {
                    return Err(CommandError::NotEnoughArgs(first, 2));
                }
                let argv = splitter(input.trim());
                match argv {
                    Some(mut args) => {
                        if args.len() < 3 {
                            return Err(CommandError::NotEnoughArgs(first, 2));
                        }
                        let extra_args: Vec<&str> = args[1].trim().split(' ').collect();
                        if extra_args.len() > 1 {
                            return Err(CommandError::TooManyArgs(first, 2))
                        } else if args[1] == "" {
                            return Err(CommandError::NotEnoughArgs(first, 2))
                        } else {
                            args[1].retain(|c| c != ' ');
                            let price = args[1].parse::<f64>();
                        match price {
                            Ok(price) => {
                                return Ok(Command::Price(args[2].clone(), price));},
                            Err(_e) => {
                                return Err(CommandError::InvalidSyntax(
                                    args[0].clone(), 
                                    args[1].clone()));
                                }
                            }
                        }
                    }
                    None => Err(CommandError::InvalidAlbum)
                }
            },
            "listen" => {
                if strings.len() == 1 {
                    return Err(CommandError::NotEnoughArgs(first, 1));
                }
                let argv = splitter(input.trim());
                match argv {
                    Some(mut args) => {
                        if args.len() < 2 {
                            return Err(CommandError::NotEnoughArgs(first, 2));
                        }
                        let extra_args: Vec<&str> = args[1].trim().split(' ').collect();
                        if extra_args.len() > 1 {
                            return Err(CommandError::TooManyArgs(first, 2));
                        } else {
                            args[1].retain(|c| c != ' ');
                            return Ok(Command::Listen(args[2].clone(), args[1].clone()));
                        }
                    }
                    None => Err(CommandError::InvalidAlbum)
                }
            },
            "query" => {
                if strings.len() == 1 {
                    return Err(CommandError::NotEnoughArgs(first, 1));
                }
                let argv = splitter(input.trim());
                match argv {
                    Some(args) => {
                        if args[1] != "" {
                            return Err(CommandError::TooManyArgs(first, 1));
                        } else if args.len() < 3 {
                            return Err(CommandError::NotEnoughArgs(first, 1));
                        } else {
                            return Ok(Command::Query(args[2].clone()));
                        }
                    }
                    None => Err(CommandError::InvalidAlbum)
                }
            },
            _ => {
                return Err(CommandError::InvalidCommand(first));
            },
        }
    }
}