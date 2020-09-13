use cursive::{
    views::{ResizedView, Dialog},
    view::SizeConstraint
};

use crate::app::database::query;

pub fn build() -> ResizedView<Dialog> {
    //TODO: Handle unwrap
    let profile = query::profile().unwrap();

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
    profile.registered,
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