use std::fs::{self, read_to_string, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use toml::{
    self, 
    value::Value, 
    map::Map,
};
use directories::ProjectDirs;

use chrono::{
    FixedOffset,
    DateTime,
    Local,
    Utc,
};
use cursive::theme::{
    Color,
    BaseColor as BC,
    PaletteColor::*,
    {BorderStyle, Palette, Theme}
};
use crate::app::{
    message::Message,
    database::query,
};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub user: User,
    pub appearance: Option<Appearance>,
}

impl Config {
    pub fn load() -> Config {
        toml::from_str(
            &read_to_string(config_file()).unwrap_or_default()
        ).unwrap_or_else(|e| {
            eprintln!("Config error: {}", e);
            std::process::exit(2);
        })
    }
    pub fn timezone() -> FixedOffset {
        let raw_tz = Config::load().user.timezone;
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
            if timezone > 14.0 || timezone < -12.0 {
                println!("Please enter a valid timezone.")
            } else {
                break;
            }
        }
        let config = Config {
            user: User { 
                username: username.trim().to_string(),
                token: token.trim().to_string(),
                timezone: timezone,
            },
            appearance: None,
        };
        if let Ok(new_config) = toml::to_string(&config) {
            match OpenOptions::new().create(true).write(true).open(&config_file()) {
                Ok(ref mut file) => {file.write(new_config.as_bytes()).unwrap();},
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
        if let None = self.folders_width {
            self.folders_width = Some(30);
        }

        if let None = self.selectcol {
            self.selectcol = Some(String::from("yellow"));
        }

        if let None = self.messagecol {
            self.messagecol = Some(gen_default_msg_cols());
        } else {
            // ensuring all values are strings
            for (k, v) in self.messagecol().iter() {
                if !v.is_str() {
                    Message::error(&format!("Incorrect type for `{}`", k))
                }
            }

        }

        if let None = self.commandcol {
            self.commandcol = Some(String::from("white"));
        }

        if let None = self.titlecol {
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

    pub fn commandcol(&self) -> Color {
        parse_colour(self.commandcol.as_ref().unwrap())
    }

    pub fn titlecol(&self) -> Color {
        parse_colour(self.titlecol.as_ref().unwrap())
    }
}

pub fn usernames_match() -> bool {
    Config::load().user.username 
    == 
    query::profile().unwrap().username
}

pub fn get_utc_now() -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        Local::now().naive_utc(),
        Utc
    )
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

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("rs", "cartoon-raccoon", "cogsy")
        .unwrap_or_else(|| panic!("Invalid home directory."))
}

pub fn config_file() -> PathBuf {
    let dirs = project_dirs();
    let mut cfgfile = PathBuf::from(dirs.config_dir());
    fs::create_dir_all(&cfgfile)
        .unwrap_or_else(|_s| panic!("Could not create config file directory"));
    cfgfile.push("config.toml");
    cfgfile
}

pub fn database_file() -> PathBuf {
    let dirs = project_dirs();
    let mut datafile = PathBuf::from(dirs.data_dir());
    fs::create_dir_all(&datafile)
        .unwrap_or_else(|_s| panic!("Could not create data file directory"));
    datafile.push("cogsy_data.db");
    datafile
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
    let mut t = Theme::default();
    t.shadow = false;
    t.borders = BorderStyle::Simple;
    t.palette = palette_gen(colours);
    return t;
}

#[cfg(test)]
mod tests {
    use dirs::home_dir;
    use super::*;
    use crate::app::App;
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
        assert_eq!(config_file(), homedir);
        homedir = home_dir().unwrap();
        homedir.push(".local/share/cogsy/cogsy_data.db");
        assert_eq!(database_file(), homedir);
    }

    #[test]
    fn check_timezone() {
        assert_eq!(
            Config::timezone(),
            FixedOffset::east(28800)
        )
    }
}