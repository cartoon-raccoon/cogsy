use cursive::{
    views::{ResizedView, Dialog},
    view::SizeConstraint
};

use crate::app::database::query;

/* 
* Designated for providing views for any action that might require a popup.
* e.g. AlbumInfo popup, multiple selection popup for :query
*/


pub fn build() -> ResizedView<Dialog> {
    //TODO: Format the Label and Formats fields properly
    let profile = query::profile().unwrap_or_default();

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