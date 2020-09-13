use clap::{
    App as Clap,
    SubCommand,
    Arg,
    ArgMatches,
};

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
        if sub_m.is_present("username") {
            println!("Updating username to {}", sub_m.value_of("username").unwrap());
        } else if sub_m.is_present("token") {
            println!("Updating token to {}", sub_m.value_of("token").unwrap());
        } else {
            println!("Updating main database");
        }
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("random") {
        if sub_m.is_present("nolog") {
            println!("Selecting random album without logging.")
        } else {
            println!("Selecting random album with logging.");
        }
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("listen") {
        println!("listening to: {}", sub_m.value_of("albumname").unwrap());
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("query") {
        if sub_m.is_present("wantlist") {
            println!("Querying wantlist for: {}", sub_m
                .value_of("wantlist")
                .unwrap_or_else(|| panic!("Album name is required.")));
        } else {
            println!("Querying collection for: {}", sub_m
                .value_of("albumname")
                .unwrap_or_else(|| panic!("Album name is required.")));
        }
        Some(())
    } else {
        None
    }
}