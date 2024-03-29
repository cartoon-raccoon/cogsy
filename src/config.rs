use std::fs::{read_to_string, OpenOptions};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

use toml::{
    self, 
    value::Value, 
    map::Map,
};

use chrono::{
    FixedOffset,
};
use cursive::theme::{
    Color,
    BaseColor as BC,
    PaletteColor::*,
    {BorderStyle, Palette, Theme}
};
use crate::app::{
    database::query::SortOrder,
    message::Message,
};
use crate::utils;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub user: User,
    pub appearance: Option<Appearance>,
}

impl Config {
    pub fn load() -> Config {
        let config_path = utils::config_file();
        if !config_path.exists() {
            Config::first_init();
        }
        toml::from_str(
            &read_to_string(config_path).unwrap_or_default()
        ).unwrap_or_else(|e| {
            eprintln!("Config error: {}", e);

            //todo: remove after upgrading to version 0.2.1
            eprint!("{}", Message::hint("hint: "));
            eprintln!("v0.2.0 marked a switch to a new config format.");
            eprintln!("See https://github.com/cartoon-raccoon/cogsy/blob/master/docs/usage.md");
            eprintln!("for more information.");

            std::process::exit(2);
        })
    }
    pub fn timezone(&self) -> FixedOffset {
        let raw_tz = self.user.timezone;
        if raw_tz < 0.0 {
            FixedOffset::west((-raw_tz * 3600.0) as i32)
        } else {
            FixedOffset::east((raw_tz * 3600.0) as i32)
        }
    }
    pub fn first_init() {
        println!("User information not initialized.");
        println!("Username:");
        let mut username = String::new();
        print!(">>> "); io::stdout().flush().unwrap();
        io::stdin().read_line(&mut username)
            .expect("Oops, could not read line.");
        println!("Token:");
        print!(">>> "); io::stdout().flush().unwrap();
        let mut token = String::new();
        io::stdin().read_line(&mut token)
            .expect("Oops, could not read line.");
        println!("Timezone:");
        let mut timezone_raw = String::new(); let mut timezone: f32;
        loop {
            io::stdin().read_line(&mut timezone_raw)
            .expect("Oops, could not read line.");
            match timezone_raw.trim().parse() {
                Ok(num) => {
                    timezone = num;
                }
                Err(_) => {
                    timezone_raw.clear();
                    println!("Please enter a valid timezone.");
                    continue;
                }
            }
            if !(-12.0..=14.0).contains(&timezone) {
                println!("Please enter a valid timezone.")
            } else {
                break;
            }
        }
        let config = Config {
            user: User { 
                username: username.trim().to_string(),
                token: token.trim().to_string(),
                timezone,
            },
            appearance: None,
        };
        if let Ok(new_config) = toml::to_string(&config) {
            match OpenOptions::new().create(true).write(true).open(&utils::config_file()) {
                Ok(ref mut file) => {file.write_all(new_config.as_bytes()).unwrap();},
                Err(_) => {panic!("Could not create config file!");}
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub username: String,
    pub token: String,
    pub timezone: f32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Appearance {
    pub format: Option<String>,
    pub sort_by: Option<String>,
    pub folders_width: Option<u32>,
    // colours
    /// The selected text colour
    pub selectcol: Option<String>,
    /// Message box colour
    pub messagecol: Option<Value>,
    /// Command line colour
    pub commandcol: Option<String>,
    /// Title colour
    pub titlecol: Option<String>,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            format: Some(String::from("{artist} - {title}")),
            sort_by: Some(String::from("default")),
            folders_width: Some(30),
            selectcol: Some(String::from("yellow")),
            messagecol: Some(gen_default_msg_cols()),
            commandcol: Some(String::from("white")),
            titlecol: Some(String::from("yellow")),
        }
    }
}

impl Appearance {

    /// Resolves all the empty (None) fields to a default colour
    pub fn resolve(&mut self) {
        if self.format.is_none() {
            self.format = Some(String::from("{artist} - {title}"));
        }

        if self.sort_by.is_none() {
            self.sort_by = Some(String::from("default"));
        }

        if self.folders_width.is_none() {
            self.folders_width = Some(30);
        }

        if self.selectcol.is_none() {
            self.selectcol = Some(String::from("yellow"));
        }

        if self.messagecol.is_none() {
            self.messagecol = Some(gen_default_msg_cols());
        } else {
            // ensuring all values are strings
            for (k, v) in self.messagecol().iter() {
                if !v.is_str() {
                    Message::error(&format!("Incorrect type for `{}`", k))
                }
            }

            if !self.messagecol_mut().contains_key("default") {
                self.messagecol_mut().insert(
                    String::from("default"), 
                    Value::String(String::from("white"))
                );
            }

            if !self.messagecol_mut().contains_key("success") {
                self.messagecol_mut().insert(
                    String::from("success"),
                    Value::String(String::from("green"))
                );
            }

            if !self.messagecol_mut().contains_key("error") {
                self.messagecol_mut().insert(
                    String::from("error"),
                    Value::String(String::from("red"))
                );
            }

            if !self.messagecol_mut().contains_key("hint") {
                self.messagecol_mut().insert(
                    String::from("hint"),
                    Value::String(String::from("yellow"))
                );
            }

            
        }
        assert!(self.messagecol().len() == 4);
        
        if self.commandcol.is_none() {
            self.commandcol = Some(String::from("white"));
        }

        if self.titlecol.is_none() {
            self.titlecol = Some(String::from("yellow"));
        }
    }

    pub fn folders(&self) -> u32 {
        self.folders_width.unwrap()
    }

    pub fn selectcol(&self) -> Color {
        parse_colour(self.selectcol.as_ref().unwrap())
    }

    pub fn messagecol(&self) -> &Map<String, Value> {
        self.messagecol.as_ref().unwrap().as_table().unwrap()
    }

    fn messagecol_mut(&mut self) -> &mut Map<String, Value> {
        self.messagecol.as_mut().unwrap().as_table_mut().unwrap()
    }

    pub fn default_col(&self) -> Color {
        parse_colour(
            self.messagecol()
                .get("default").unwrap()
                .as_str().unwrap()
        )
    }

    pub fn success_col(&self) -> Color {
        parse_colour(
            self.messagecol()
                .get("success").unwrap()
                .as_str().unwrap()
        )
    }

    pub fn hint_col(&self) -> Color {
        parse_colour(
            self.messagecol()
                .get("hint").unwrap()
                .as_str().unwrap()
        )
    }

    pub fn error_col(&self) -> Color {
        parse_colour(
            self.messagecol()
                .get("error").unwrap()
                .as_str().unwrap()
        )
    }

    pub fn commandcol(&self) -> Color {
        parse_colour(self.commandcol.as_ref().unwrap())
    }

    pub fn titlecol(&self) -> Color {
        parse_colour(self.titlecol.as_ref().unwrap())
    }

    pub fn sort_by(&self) -> SortOrder {
        use SortOrder::*;
        match self.sort_by.as_ref().unwrap().as_str() {
            "default" => Default,
            "id" => Id,
            "title" => Title,
            "artist" => Artist,
            "year" => Year,
            "date" => Date,
            _ => Default,
        }
    }
}

fn parse_colour(color: &str) -> Color {
    match color {
        "black"     => Color::Dark(BC::Black),
        "red"       => Color::Dark(BC::Red),
        "green"     => Color::Dark(BC::Green),
        "yellow"    => Color::Dark(BC::Yellow),
        "blue"      => Color::Dark(BC::Blue),
        "magenta"   => Color::Dark(BC::Magenta),
        "cyan"      => Color::Dark(BC::Cyan),
        "white"     => Color::Dark(BC::White),
        "brblack"   => Color::Light(BC::Black),
        "brred"     => Color::Light(BC::Red),
        "brgreen"   => Color::Light(BC::Green),
        "bryellow"  => Color::Light(BC::Yellow),
        "brblue"    => Color::Light(BC::Blue),
        "brmagenta" => Color::Light(BC::Magenta),
        "brcyan"    => Color::Light(BC::Cyan),
        "brwhite"   => Color::Light(BC::White),
        _           => Color::Light(BC::Yellow),
    }
}

fn gen_default_msg_cols() -> Value {
    let mut table = Map::with_capacity(3);
                
    table.insert(
        String::from("default"), 
        Value::String(String::from("white"))
    );

    table.insert(
        String::from("error"),
        Value::String(String::from("red"))
    );

    table.insert(
        String::from("success"),
        Value::String(String::from("green"))
    );

    table.insert(
        String::from("hint"),
        Value::String(String::from("yellow"))
    );

    Value::Table(table)
}

impl Default for Config {
    fn default() -> Config {
        Config {
            user: User {
                username: String::from("null"),
                token: String::from("null"),
                timezone: 0.0,
            },
            appearance: Some(Appearance::default()),
        }
    }
}

#[derive(Deserialize)]
pub struct ColourScheme {

}

pub fn palette_gen(colours: &mut Appearance) -> Palette {
    colours.resolve();

    let mut p = Palette::default();
    p[Background] = Color::TerminalDefault;
    p[Shadow] = Color::TerminalDefault;
    p[View] = Color::TerminalDefault;
    p[Primary] = Color::TerminalDefault;

    // Command line text colour
    p[Secondary] = colours.commandcol();

    p[Tertiary] = Color::TerminalDefault;

    // Popup title colours
    p[TitlePrimary] = colours.titlecol();
    p[Highlight] = Color::TerminalDefault;
    p[HighlightInactive] = Color::TerminalDefault;
    p[HighlightText] = colours.selectcol();

    p
}

pub fn theme_gen(colours: &mut Appearance) -> Theme {
    Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: palette_gen(colours),
    }
}

#[cfg(test)]
mod tests {
    use dirs::home_dir;
    use super::*;
    use crate::app::App;
    use crate::CONFIG;
    #[test] //it fails lmao but code works correctly
    fn check_config_loads_correctly() {
        let testcfg = Config::load();
        let checkcfg = App::initialize();
        println!("{}", testcfg.user.token);
        assert_eq!(testcfg.user.username, checkcfg.user_id);
        assert_eq!(testcfg.user.timezone, 8.0);
    }

    #[test]
    fn check_filepaths() {
        let mut homedir = home_dir().unwrap();
        homedir.push(".config/cogsy/config.toml");
        assert_eq!(utils::config_file(), homedir);
        homedir = home_dir().unwrap();
        homedir.push(".local/share/cogsy/cogsy_data.db");
        assert_eq!(utils::database_file(), homedir);
    }

    #[test]
    fn check_timezone() {
        assert_eq!(
            CONFIG.timezone(),
            FixedOffset::east(28800)
        )
    }
}