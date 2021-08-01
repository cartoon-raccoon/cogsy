use std::{io, process};
use clap::{
    App as Clap,
    SubCommand,
    Arg,
    ArgMatches,
};
use crate::CONFIG;
use crate::utils;
use crate::app::{
    ListenLogEntry,
    Release,
    App,
    update::{self, UpdateError},
    database::{
        admin,
        query::{self, QueryType},
        update as dbupdate,
        purge,
    },
    message::{Message, MessageKind},
    csv,
};

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
                .help("Toggle verbose output when updating"))
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
                .value_name("TOKEN")
                .help("Updates the token"))
            .arg(Arg::with_name("csv")
                .short("c")
                .long("csv")
                .takes_value(true)
                .value_name("CSV")
                .multiple(true)
                .validator(validate_csv)
                .max_values(2)
                .help("Select which parts to read from a csv file"))
            .arg(Arg::with_name("profile")
                .short("P")
                .long("profile")
                .takes_value(false)
                .help("Updates the profile only"))
            .arg(Arg::with_name("wantlist")
                .short("W")
                .long("wantlist")
                .takes_value(false)
                .help("Updates the wantlist only"))
            .arg(Arg::with_name("collection")
                .short("C")
                .long("collection")
                .takes_value(false)
                .help("Updates the collection only"))
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
        .subcommand(SubCommand::with_name("database")
            .about("Options for database administration.")
            .arg(Arg::with_name("reset")
                .short("r")
                .long("reset")
                .takes_value(false)
                .help("Performs database reset.")
            )
            .arg(Arg::with_name("orphan")
                .short("o")
                .long("orphan")
                .takes_value(false)
                .help("Performs orphan table removal.")
            )
            .arg(Arg::with_name("check")
                .short("c")
                .long("check")
                .takes_value(false)
                .help("Performs database integrity check.")
            )
        )   
}

// valid argument is: wantlist=<path> or collection=<path>
// if the argument is not wantlist or collection, or no path is
// provided, an error is returned
fn validate_csv(s: String) -> Result<(), String> {
    if s.starts_with("wantlist") || s.starts_with("collection") {
        if s.split('=').count() != 2 {
            Err(String::from("invalid number of argument values"))
        } else {
            Ok(())
        }
    } else {
        Err(String::from("csv value must be `wantlist` or `collection`"))
    }
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
    let verbose = sub_m.is_present("verbose");

    let (one, two) = get_csv(sub_m);
    let csvs = CsvUpdate::from_tuple((
        one.map(|s| UpdateType::from_arg(s)),
        two.map(|s| UpdateType::from_arg(s)),
    ));

    if sub_m.is_present("username") {
        println!("Sorry, in-app username updates are unsupported at this time.");
        return Some(3)
    } else if sub_m.is_present("token") {
        println!("Sorry, in-app token updates are unsupported at this time.");
        return Some(3)
    } else {
        let mut ran_update = false;

        if sub_m.is_present("profile") {
            ran_update = true;
            println!("{}",
            Message::set("Beginning profile update.", MessageKind::Info)
            );
            if let Err(e) = update::profile(&app.user_id, &app.token, true) {
                Message::error(&e.to_string());
            }
        }
        if sub_m.is_present("wantlist") {
            ran_update = true;
            println!("{}",
            Message::set("Beginning wantlist update.", MessageKind::Info)
            );
            if let Some(path) = csvs.wantlist.as_ref() {
                println!("Updating wantlist from CSV file at path `{}`.", path);
                if let Err(e) = csv::update_want(path) {
                    Message::error(&e.to_string());
                } else {
                    println!("{}", Message::success("Update from CSV successful."));
                }
            } else if let Err(e) = update::wantlist(&app.user_id, &app.token, true, verbose) {
                Message::error(&e.to_string());
            }
        }
        if sub_m.is_present("collection") {
            ran_update = true;
            println!("{}",
            Message::set("Beginning collection update.", MessageKind::Info)
            );
            if let Some(path) = csvs.collection.as_ref() {
                println!("Updating collection from CSV file at path `{}`.", path);
                if let Err(e) = csv::update_coll(path) {
                    Message::error(&e.to_string());
                } else {
                    println!("{}", Message::success("Update from CSV successful."));
                }
            } else if let Err(e) = update::collection(&app.user_id, &app.token, true, verbose) {
                Message::error(&e.to_string());
            }
        }
        if ran_update {return Some(0)}
        println!("{}",
            Message::set("Beginning full database update.", MessageKind::Info)
        );
        let updates = (csvs.wantlist.as_ref(), csvs.collection.as_ref());

        // if either is to be updated from CSV
        if csvs.wantlist.is_some() || csvs.collection.is_some() {
            if let Err(e) = update::profile(&app.user_id, &app.token, true) {
                Message::error(&e.to_string())
            }
            println!("{}\n", Message::success("Profile update successful."));
        }

        match updates {
            (Some(s1), Some(s2)) => {
                println!("{}", Message::info(
                    format!(
                        "Updating wantlist and collection from CSV files at:\n - {}\n - {}",
                        s1, s2
                    )
                ));
                if let Err(e) = csv::full_update(s1, s2) {
                    Message::error(&e.to_string());
                }
                println!("{}", Message::success("Full CSV update successful."));
            }
            (Some(s), None) => {
                println!("{}", Message::info(format!("Updating wantlist from CSV file at {}.", s)));
                if let Err(e) = csv::update_want(s) {
                    Message::error(&e.to_string());
                }
                println!("{}\n", Message::success("Wantlist update successful."));
                println!("{}", Message::info("Updating collection from Discogs."));
                if let Err(e) = update::collection(&app.user_id, &app.token, true, verbose) {
                    Message::error(&e.to_string());
                }
                println!("{}", Message::success("Collection update successful."))
            }
            (None, Some(s)) => {
                println!("{}", Message::info("Updating wantlist from Discogs."));
                if let Err(e) = update::wantlist(&app.user_id, &app.token, true, verbose) {
                    Message::error(&e.to_string());
                }
                println!("{}\n", Message::success("Wantlist update successful."));
                println!("{}", Message::info(format!("Updating collection from CSV file at {}", s)));
                if let Err(e) = csv::update_coll(s) {
                    Message::error(&e.to_string());
                }
                println!("{}", Message::success("Collection update from CSV successful."))
            }
            (None, None) => {
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
        }
    }
    Some(0)
}

// ! Warning: spaghetti code ahead

/// enum type used when csv updating is required.
#[derive(Clone, Copy, Debug)]
enum UpdateType<'a> {
    Collection(&'a str),
    Wantlist(&'a str),
}

impl<'a> UpdateType<'a> {
    fn from_arg(arg: &'a str) -> Self {
        let mut split = arg.split('=');

        let place = split.next();

        if place == Some("wantlist") {
            Self::Wantlist(split.next().expect("No value provided"))
        } else if place == Some("collection") {
            Self::Collection(split.next().expect("No value provided"))
        } else {
            unreachable!("invalid arg should've be caught by Clap")
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Collection(s) => s,
            Self::Wantlist(s) => s,
        }
    }

    fn is_collection(&self) -> bool {
        matches!(self, Self::Collection(_))
    }

    fn is_wantlist(&self) -> bool {
        matches!(self, Self::Wantlist(_))
    }
}

#[derive(Clone, Debug)]
struct CsvUpdate {
    wantlist: Option<String>,
    collection: Option<String>,
}

impl CsvUpdate {
    fn from_tuple(
        tup: (Option<UpdateType>, Option<UpdateType>)
    ) -> Self {
        Self {
            wantlist: if let Some(udt) = tup.0 {
                if udt.is_wantlist() {
                    Some(udt.as_str().into())
                } else if udt.is_collection() {
                    if let Some(udt) = tup.1 {
                        if udt.is_wantlist() {
                            Some(udt.as_str().into())
                        } else {None}
                    } else {None}
                } else {None}
            } else if let Some(udt) = tup.1 {
                if udt.is_wantlist() {
                    Some(udt.as_str().into())
                } else {None}
            } else {None},
            collection: if let Some(udt) = tup.1 {
                if udt.is_collection() {
                    Some(udt.as_str().into())
                } else if udt.is_wantlist() {
                    if let Some(udt) = tup.0 {
                        if udt.is_collection() {
                            Some(udt.as_str().into())
                        } else {None}
                    } else {None}
                } else {None}
            } else if let Some(udt) = tup.0 {
                if udt.is_collection() {
                    Some(udt.as_str().into())
                } else {None}
            } else {None},
        }
    }
}

// returns in the sequence: wantlist, collection
fn get_csv<'a>(sub_m: &'a ArgMatches) -> (Option<&'a str>, Option<&'a str>) {
    if let Some(mut vals) = sub_m.values_of("csv") {
        (vals.next(), vals.next())
    } else {
        (None, None)
    }
}

//* End of spaghetti code

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

    match query::release(&album, QueryType::Collection) {
        Ok(results) => {
            match results.len() {
                1 => {
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
                }
                n if n > 1 => {
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
                            release.formats.join(", "),
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
                }
                _ => {
                    println!("Unable to find results for `{}`", album);
                    return Some(1)
                }
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
            println!("{} Album name is required.", Message::set("Error:", MessageKind::Error));
            process::exit(1);
        }).to_string()
        .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

        querytype = QueryType::Wantlist;
        println!("Querying wantlist for: {}", query);
    } else {
        query = sub_m.value_of("albumname")
        .unwrap_or_else(|| {
            println!("{} Album name is required.", Message::set("Error:", MessageKind::Error));
            process::exit(1);
        }).to_string()
        .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

        querytype = QueryType::Collection;
        println!("Querying collection for: {}\n", query);
    }
    results = match query::release(
        &query, querytype
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
        println!("{}", release);
    }
    Some(0)
}

pub fn handle_database(sub_m: &ArgMatches) -> Option<i32> {
    if sub_m.is_present("reset") {
        if sub_m.is_present("orphan") || sub_m.is_present("check") {
            return database_arg_error()
        }
        return handle_reset()
    } else if sub_m.is_present("orphan") {
        if sub_m.is_present("reset") || sub_m.is_present("check") {
            return database_arg_error()
        }
        return handle_orphans()
    } else if sub_m.is_present("check") {
        if sub_m.is_present("orphan") || sub_m.is_present("reset") {
            return database_arg_error()
        }
        return handle_check()
    } else {

    }
    Some(0)
}

fn handle_reset() -> Option<i32> {
    println!("{}", Message::set("Resetting database.", MessageKind::Info));
    match purge::complete() {
        Ok(_) => {
            println!("Database purged. Pulling data from Discogs...");
            match update::full(&CONFIG.user.username, &CONFIG.user.token, true, false) {
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

fn handle_check() -> Option<i32> {
    println!("{}", Message::info("Performing database integrity check."));
    match admin::check_integrity() {
        Ok(_) => {
            println!("{}", Message::success("No errors found."));
            Some(0)
        }
        Err(e) => {
            eprintln!("error: {}", e);
            Some(2)
        }
    }
}

fn handle_orphans() -> Option<i32> {
    println!("{}", Message::info("Removing orphan tables."));
    match admin::remove_orphans() {
        Ok(_) => {
            println!("{}", Message::success("Removal successful."));
            Some(0)
        }
        Err(e) => {
            eprintln!("error: {}", e);
            Some(2)
        }
    }
}

fn database_arg_error() -> Option<i32> {
    eprintln!("error: too many commands");
    Some(1)
}

fn db_error_msg() {
    eprintln!("This is a database error and may be a bug.");
    eprintln!("Please file an issue on GitHub with this error message.");
}