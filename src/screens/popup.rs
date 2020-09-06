use cursive::views::*;
use cursive::view::SizeConstraint;

use crate::app::Release;

/* 
* Designated for providing views for any action that might require a popup.
* e.g. AlbumInfo popup, multiple selection popup for :query
*/


pub fn albuminfo(release: &Release) -> ResizedView<Dialog> {
    //TODO: Format the Label and Formats fields properly
    let content = String::from(format!("
    Artist: {}

    Year Released: {}

    Labels: {:?}

    Formats: {:?}

    Date Added: {}",
    release.artist,
    release.year,
    release.labels,
    release.formats,
    release.date_added
    ));

    let screen = ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        Dialog::text(content)
            .title(release.title.clone())
    );
    screen
}