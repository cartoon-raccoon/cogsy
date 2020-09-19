//* this module only formats text for commandline usage.
//* i'm still trying to figure out how to format text for the tui.

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
}

impl std::default::Default for Message {
    fn default() -> Self {
        Message {
            msg: String::from("what"),
            kind: MessageKind::Info,
        }
    }
}