pub mod impl_app;
pub mod database;
pub mod request;
pub mod message;
pub mod response;

use message::Message;

pub struct App{
    pub user_id: String,
    pub token: String,
    pub message: Message,
}