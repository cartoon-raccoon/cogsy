use std::io;
use clap::{
    App as Clap,
    SubCommand,
    Arg,
    ArgMatches,
};
use crate::utils::{self, Config};
use crate::app::{
    ListenLogEntry,
    Release,
    App,
    update,
    database::{
        query::{self, QueryType},
        update as dbupdate,
    }
};
use crate::screens::popup::format_vec;

pub fn init<'a>() -> Clap<'a, 'a> {
    Clap::new("cogsy")
        .author("cartoon.raccoon")
        .version("1.0.0")
        .about("A command line Discogs client written in Rust")
        .subcommand(SubCommand::with_name("update")
            .about("Updates the cogsy database.")
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
}

pub fn parse_and_execute(clapapp: ArgMatches) -> Option<()> {
    if let Some(sub_m) = clapapp.subcommand_matches("update") {
        let app = App::initialize();
        if sub_m.is_present("username") {
            println!("Updating username to {}", sub_m.value_of("username").unwrap());
        } else if sub_m.is_present("token") {
            println!("Updating token to {}", sub_m.value_of("token").unwrap());
        } else {
            println!("Beginning full database update.");
            match update::full(app.user_id, app.token, true, false) {
                Ok(()) => {}
                Err(e) => {eprintln!("{}", e)}
            }
        }
        Some(())
    //TODO: Implement this
    } else if let Some(sub_m) = clapapp.subcommand_matches("random") {
        if sub_m.is_present("nolog") {
            println!("Selecting random album without logging.");
            match query::random() {
                Ok(random) => {
                    println!("You should play `{}.`", random.title);
                }
                Err(e) => {
                    eprintln!("Oops: {}", e);
                }
            }
        } else {
            println!("Selecting random album with logging.");
            match query::random() {
                Ok(random) => {
                    let time_now = utils::get_utc_now();
                    let entry = ListenLogEntry {
                        id: random.id,
                        title: random.title.clone(),
                        time: time_now,
                    };
                    match dbupdate::listenlog(entry) {
                        Ok(()) => {
                            println!("You should play `{}`.", random.title);
                        }
                        Err(e) => {
                            eprintln!("Oops: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Oops: {}", e);
                }
            }
        }
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("listen") {
        let album = sub_m.value_of("albumname").unwrap().to_string();
        match query::release(album.clone(), QueryType::Collection) {
            Ok(results) => {
                if results.len() > 1 {
                    println!("Multiple results for `{}`, pick one:", album);
                    for release in &results {
                        println!(
                            "[{}]: {} - {} ({})",
                            results.iter().position(|x| x == release).unwrap() + 1,
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
                            Err(_) => {println!("Invalid input!"); continue}
                        };
                        if choice <= results.len() {
                            let time_now = utils::get_utc_now();
                            let entry = ListenLogEntry {
                                id: results[choice - 1].id,
                                title: results[choice - 1].title.clone(),
                                time: time_now,
                            };
                            match dbupdate::listenlog(entry) {
                                Ok(()) => {println!("Listening to `{}` by {}", 
                                    results[choice - 1].title, 
                                    results[choice - 1].artist);
                                }
                                Err(e) => {eprintln!("{}", e);}
                            }
                            break;
                            } else {
                            println!("Please select a valid choice.");
                        }
                    }
                } else if results.len() == 1 {
                    let time_now = utils::get_utc_now();
                    let entry = ListenLogEntry {
                        id: results[0].id,
                        title: results[0].title.clone(),
                        time: time_now,
                    };
                    match dbupdate::listenlog(entry) {
                        Ok(()) => {println!("Listening to `{}` by {}", 
                            results[0].title, results[0].artist);}
                        Err(e) => {eprintln!("{}", e);}
                    }
                } else {
                    println!("Unable to find results for `{}`", album);
                }  
            },
            Err(e) => {eprintln!("{}", e);}
        }  
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("query") {
        let results: Vec<Release>;
        let query: String;
        let querytype: QueryType;
        //TODO: Streamline this wet-ass code
        if sub_m.is_present("wantlist") {
            query = sub_m.value_of("wantlist")
            .unwrap_or_else(|| panic!("Album name is required.")).to_string();
            querytype = QueryType::Wantlist;
            println!("Querying wantlist for: {}", query);
        } else {
            query = sub_m.value_of("albumname")
            .unwrap_or_else(|| panic!("Album name is required.")).to_string();
            querytype = QueryType::Collection;
            println!("Querying collection for: {}\n", query);
        }
        results = query::release(
            query.clone(), querytype
        ).unwrap_or_else(|e| {eprintln!("Oops: {}", e); Vec::new()});
        if results.len() > 1 {
            println!("Multiple results for `{}`:\n", query)
        }
        if results.is_empty() {
            println!("Nothing found for `{}`.", query);
        }
        for release in results {
            let display_time = release.date_added
            .with_timezone(&Config::timezone());

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
        Some(())
    } else {
        None
    }
}