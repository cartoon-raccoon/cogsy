pub mod impl_app;
pub mod database;
pub mod request;
pub mod message;
pub mod response;

use message::Message;
use crate::collection::Collection;
use crate::wantlist::Wantlist;

//#[derive(Debug, Clone)]
pub struct App{
    pub user_id: String,
    pub token: String,
    pub message: Message,
    pub collection: Collection,
    pub wantlist: Wantlist,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub year: u64,
    pub labels: Vec<String>,
    pub formats: Vec<String>,
    pub date_added: String,
}