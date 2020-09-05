use cursive::theme::{BaseColor, Color, ColorStyle, Style};
use cursive::utils::span::SpannedString;

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
}

fn get_style(item: MessageKind) -> Style {
    match item {
        MessageKind::Error => {
            let mut error_style = Style::none();
            error_style.color = Some(ColorStyle::new(
                Color::Dark(BaseColor::Red),
                Color::TerminalDefault
            ));

            return error_style;
        },
        MessageKind::Info => {
            let mut info_style = Style::none();
            info_style.color = 
            Some(ColorStyle::new(
                Color::Dark(BaseColor::Yellow),
                Color::TerminalDefault
            ));

            return info_style;
        }
        MessageKind::Hint => {
            let mut hint_style = Style::none();
            hint_style.color = 
            Some(ColorStyle::new(
                Color::Dark(BaseColor::Green),
                Color::TerminalDefault
            ));

            return hint_style;
        },
    }
}

impl From<MessageKind> for Color {
    fn from(item: MessageKind) -> Self {
        match item {
            MessageKind::Error => Color::Dark(BaseColor::Red),
            MessageKind::Info => Color::Dark(BaseColor::Yellow),
            MessageKind::Hint => Color::Dark(BaseColor::White),
        }
    }
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

#[allow(dead_code)]
pub struct Message {
    pub msg: String,
    pub kind: MessageKind,
}

#[allow(dead_code)]
impl Message {
    pub fn contents(&self) -> &String {
        &self.msg
    }
    pub fn kind(&self) -> MessageKind {
        self.kind
    }
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
            msg: String::from("what"), //StyledString::styled("", get_style(MessageKind::Info)),
            kind: MessageKind::Info,
        }
    }
}