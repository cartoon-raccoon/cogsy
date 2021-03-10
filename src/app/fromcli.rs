use std::io;
use std::process;
use clap::{
    App as Clap,
    SubCommand,
    Arg,
    ArgMatches,
};
use crate::CONFIG;
use crate::config::Config;
use crate::utils;
use crate::app::{
    ListenLogEntry,
    Release,
    App,
    update::{self, UpdateError},
    database::{
        query::{self, QueryType},
        update as dbupdate,
        purge,
    },
    message::{Message, MessageKind},
};
use crate::screens::popup::format_vec;

pub fn init<'a>() -> Clap<'a, 'a> {
    Clap::new("cogsy")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("A command line Discogs client written in Rust")
        .subcommand(SubCommand::with_name("update")
            .about("Updates the cogsy database.")
            .arg(Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .help("Toggle verbose output when updating."))
            .arg(Arg::with_name("username")
                .short("u")
                .long("username")
                .takes_value(true)
                .value_name("USERNAME")
                .help("Updates the username"))
            .arg(Arg::with_name("token")
                .short("t")
                .long("token")
                .takes_value(true)
                .value_name("token")
                .help("Updates the token")
            )
        )
        .subcommand(SubCommand::with_name("random")
            .about("Select a random song to play.")
            .arg(Arg::with_name("nolog")
                .short("n")
                .long("nolog")
                .takes_value(false)
                .required(false)
                .help("The album won't be logged to the listening log.")
            )
        )
        .subcommand(SubCommand::with_name("listen")
            .about("Log an album that you're listening to.")
            .arg(Arg::with_name("albumname")
                .required(true)
                .help("The name of the album you want to play.")
            )
        )
        .subcommand(SubCommand::with_name("query")
            .about("Query the database for an album.")
            .arg(Arg::with_name("wantlist")
                .short("w")
                .long("wantlist")
                .takes_value(true)
                .help("Use this switch to query from the wantlist.")
            )
            .arg(Arg::with_name("albumname")
                .help("The name of the album you want to query.")
            )
        )
        .subcommand(SubCommand::with_name("reset")
            .about("Reset the entire database.")
        )   
}

//* CLI Mode Exit codes:
//* 0: All good
//* 1: Incorrect input from user
//* 2: Internal app error
//* 3: Unsupported feature

pub fn parse_and_execute(clapapp: ArgMatches, app: &App) -> Option<i32> {
    if let Some(sub_m) = clapapp.subcommand_matches("update") {
        handle_update(sub_m , app)
    } else if let Some(sub_m) = clapapp.subcommand_matches("random") {
        handle_random(sub_m)
    } else if let Some(sub_m) = clapapp.subcommand_matches("listen") {
        handle_listen(sub_m)
    } else if let Some(sub_m) = clapapp.subcommand_matches("query") {
        handle_query(sub_m)
    } else {
        None
    }
}

fn handle_update(sub_m: &ArgMatches, app: &App) -> Option<i32> {
    if sub_m.is_present("username") {
        println!("Sorry, in-app username updates are unsupported at this time.");
        return Some(3)
    } else if sub_m.is_present("token") {
        println!("Sorry, in-app token updates are unsupported at this time.");
        return Some(3)
    } else {
        println!("{}",
            Message::set("Beginning full database update.", MessageKind::Info)
        );
        let verbose = if sub_m.is_present("verbose") {true} else {false};
        if verbose {
            println!("{}", Message::info("Info: verbose output not yet implemented."));
        }
        match update::full(&app.user_id, &app.token, true, verbose) {
            Ok(()) => {
                println!("{}", Message::success("Database update successful."));
            }
            Err(e) => {
                eprintln!("\n{}", e);
                if let UpdateError::DBWriteError(_) = e {
                    db_error_msg();
                }
                return Some(2)
            }
        }
    }
    Some(0)
}

fn handle_random(sub_m: &ArgMatches) -> Option<i32> {
    if sub_m.is_present("nolog") {
        println!("{}", 
            Message::info("Selecting random album without logging.")
        );
        match query::random() {
            Ok(random) => {
                println!("You should play `{}.`", random.title);
            }
            Err(e) => {
                eprintln!("{}", e);
                db_error_msg();
                return Some(2)
            }
        }
    } else {
        println!("{}", 
            Message::info("Selecting random album with logging.")
        );
        match query::random() {
            Ok(random) => {
                let time_now = utils::get_utc_now();
                let entry = ListenLogEntry {
                    id: random.id,
                    title: &random.title,
                    time: time_now,
                };
                match dbupdate::listenlog(entry) {
                    Ok(()) => {
                        println!("You should play `{}`.", random.title);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        db_error_msg();
                        return Some(2)
                    }
                }
            }
            Err(e) => {
                eprintln!("Oops: {}", e);
                return Some(2)
            }
        }
    }
    Some(0)
}

fn handle_listen(sub_m: &ArgMatches) -> Option<i32> {
    let album = sub_m.value_of("albumname").unwrap().to_string()
        .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

        match query::release(album.clone(), QueryType::Collection) {
            Ok(results) => {
                if results.len() > 1 {
                    println!("{}",
                        Message::info(
                            &format!("Multiple results for `{}`, pick one:", album),
                        )
                    );
                    for (i, release) in results.iter().enumerate() {
                        println!(
                            "[{}]: {} - {} ({})",
                            i + 1,
                            release.artist,
                            release.title,
                            format_vec(&release.formats),
                        );
                    }
                    loop {
                        let mut answer = String::new();
                        io::stdin().read_line(&mut answer)
                            .expect("Oops, could not read line.");
                        let choice: usize = match answer.trim().parse() {
                            Ok(num) => num,
                            Err(_) => {println!("{}",
                                Message::set("Invalid input!", MessageKind::Error)
                            ); continue}
                        };
                        if choice <= results.len() {
                            let time_now = utils::get_utc_now();
                            let entry = ListenLogEntry {
                                id: results[choice - 1].id,
                                title: &results[choice - 1].title,
                                time: time_now,
                            };
                            match dbupdate::listenlog(entry) {
                                Ok(()) => {println!("Listening to `{}` by {}", 
                                    results[choice - 1].title, 
                                    results[choice - 1].artist);
                                }
                                Err(e) => {
                                    eprintln!("Database error: {}", e);
                                    db_error_msg();
                                    return Some(2)
                                }
                            }
                            break;
                            } else {
                            println!("{}",
                                Message::set("Please select a valid choice.", MessageKind::Error)
                            );
                        }
                    }
                } else if results.len() == 1 {
                    let time_now = utils::get_utc_now();
                    let entry = ListenLogEntry {
                        id: results[0].id,
                        title: &results[0].title,
                        time: time_now,
                    };
                    match dbupdate::listenlog(entry) {
                        Ok(()) => {println!("Listening to `{}` by {}", 
                            results[0].title, results[0].artist);}
                        Err(e) => {
                            eprintln!("{}", e);
                            db_error_msg();
                            return Some(2)
                        }
                    }
                } else {
                    println!("Unable to find results for `{}`", album);
                    return Some(1)
                }  
            },
            Err(e) => {
                eprintln!("{}", e);
                db_error_msg();
                return Some(2)
            }
        }  
        Some(0)
}

fn handle_query(sub_m: &ArgMatches) -> Option<i32> {
    let results: Vec<Release>;
    let query: String;
    let querytype: QueryType;
    //TODO: Streamline this wet-ass code
    if sub_m.is_present("wantlist") {
        query = sub_m.value_of("wantlist")
        .unwrap_or_else(|| {
            println!("{} {}", Message::set("Error:", MessageKind::Error), "Album name is required.");
            process::exit(1);
        }).to_string()
        .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

        querytype = QueryType::Wantlist;
        println!("Querying wantlist for: {}", query);
    } else {
        query = sub_m.value_of("albumname")
        .unwrap_or_else(|| {
            println!("{} {}", Message::set("Error:", MessageKind::Error), "Album name is required.");
            process::exit(1);
        }).to_string()
        .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

        querytype = QueryType::Collection;
        println!("Querying collection for: {}\n", query);
    }
    results = match query::release(
        query.clone(), querytype
    ) {
        Ok(queryr) => queryr,
        Err(e) => {
            eprintln!("Database error: {}", e);
            db_error_msg();
            return Some(2)
        }
    };
    if results.len() > 1 {
        println!("Multiple results for `{}`:\n", query)
    }
    if results.is_empty() {
        println!("Nothing found for `{}`.", query);
        return Some(1)
    }
    for release in results {
        let display_time = release.date_added
        .with_timezone(&CONFIG.timezone());

        println!(
            "{} by {}:\nReleased: {}\nLabels: {}\nFormats: {}\nAdded: {}\n",
            release.title,
            release.artist,
            release.year,
            format_vec(&release.labels),
            format_vec(&release.formats),
            display_time.format("%A %d %m %Y %R"),
        )
    }
    Some(0)
}

pub fn handle_reset(config: &Config) -> Option<i32> {
    println!("{}", Message::set("Resetting database.", MessageKind::Info));
    match purge::complete() {
        Ok(_) => {
            println!("Database purged. Pulling data from Discogs...");
            match update::full(&config.user.username, &config.user.token, true, false) {
                Ok(()) => {}
                Err(e) => {
                    println!("\n{}", e);
                    if let UpdateError::DBWriteError(_) = e {
                        db_error_msg();
                    }
                    std::fs::remove_file(utils::database_file()).unwrap();
                    return Some(2)
                }
            }
            println!("{}", Message::set("Successfully reset database.", MessageKind::Success));
            Some(0)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Some(2)
        }
    }
}

fn db_error_msg() {
    eprintln!("This is a database error and is likely a bug.");
    eprintln!("Please file an issue on GitHub with this error message.");
}