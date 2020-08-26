pub mod impl_app;
pub mod database;
pub mod request;
pub mod message;
pub mod response;

pub struct App{
    user_id: String,
    token: String,
}