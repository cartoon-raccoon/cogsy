use cursive::{
    views::{ResizedView, Dialog},
    view::SizeConstraint
};

use crate::utils::Config;
use crate::app::database::query;

pub fn build() -> ResizedView<Dialog> {
    //TODO: Handle unwrap
    let profile = query::profile().unwrap();

    //* timezone currently hardcoded
    //TODO: Add a function to utils to return the user's set timezone
    let display_time = profile.registered
    .with_timezone(&Config::timezone());

    let content = String::from(format!("
    Username: {}
    Name: {}

    Date Registered: {}

    No. of Listings: {}
    Collection Size: {}
    Wantlist Size: {}
    
    Releases rated: {}
    Average rating: {}",
    profile.username,
    profile.real_name,
    display_time,
    profile.listings,
    profile.collection,
    profile.wantlist,
    profile.rated,
    profile.average_rating,
    ));

    let screen = ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        Dialog::text(content)
            .title(format!("Profile: {}", 
            profile.username))
    );
    screen
}