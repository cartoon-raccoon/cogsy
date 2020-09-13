use cursive::{
    views::{ResizedView, Dialog},
    view::SizeConstraint
};

use crate::utils::Config;
use crate::app::database::query;

pub fn build() -> ResizedView<Dialog> {
    //TODO: Implement default for DateTime so can call default if query fails
    let profile = query::profile().unwrap_or_else(
        |_s| panic!("Fatal: Could not load profile from database.")
    );

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
    display_time.format("%A %d %m %Y %R"),
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