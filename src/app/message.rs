use ansi_term::Colour::{Red, Green, Yellow};
use ansi_term::Style;

/* 
The idea is that this module processes a function that returns
a SpannedString to insert as input to the TextView (commandline)

The base was shamelessly stolen from github.com/NerdyPepper/dijo
but heavily adapted for my needs
*/

#[derive(Debug, Clone, Copy)]
pub enum MessageKind {
    Error,
    Info,
    Hint,
    Success,
}

impl<T> From<T> for Message
where
    T: AsRef<str>,
{
    fn from(item: T) -> Self {
        return Message {
            msg: item.as_ref().to_string(),
            kind: MessageKind::Info,
        };
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.kind {
            MessageKind::Error => {
                write!(f, "{}", Red.bold().paint(&self.msg))
            }
            MessageKind::Hint => {
                write!(f, "{}", Yellow.paint(&self.msg))
            }
            MessageKind::Info => {
                write!(f, "{}", Style::new().bold().paint(&self.msg))
            }
            MessageKind::Success => {
                write!(f, "{}", Green.bold().paint(&self.msg))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub msg: String,
    pub kind: MessageKind,
}

impl Message {
    pub fn set(text: &str, kind: MessageKind) -> Self {
        Message {
            msg: text.to_string(),
            kind: kind,
        }
    }

    pub fn error(text: &str) -> ! {
        let msg = Message {
            msg: text.into(),
            kind: MessageKind::Error,
        };

        eprintln!("{}", msg);
        std::process::exit(2)
    }
}

impl std::default::Default for Message {
    fn default() -> Self {
        Message {
            msg: String::from("what"),
            kind: MessageKind::Info,
        }
    }
}

pub mod msgbox {
    use cursive::theme::{Color, BaseColor as Bc};

    pub const ERROR: Color = Color::Dark(Bc::Red);
    pub const DEFAULT: Color = Color::Dark(Bc::White);
    pub const SUCCESS: Color = Color::Dark(Bc::Green);
    pub const HINT: Color = Color::Dark(Bc::Yellow);

}