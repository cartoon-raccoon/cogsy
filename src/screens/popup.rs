use std::collections::HashSet;

use cursive::{
    views::{
        ResizedView, 
        Dialog,
        SelectView,
    },
    view::SizeConstraint
};

use crate::utils::{self, Config};
use crate::app::{
    Release,
    ListenLogEntry,
};
use crate::app::database::update;

/* 
* Designated for providing views for any action that might require a popup.
* e.g. AlbumInfo popup, multiple selection popup for :query
*/


pub fn albuminfo(release: &Release) -> ResizedView<Dialog> {
    //TODO: Format the Label and Formats fields properly
    let set: HashSet<_> = release.labels.clone().drain(..).collect();
    let mut labels: Vec<String> = Vec::new();
    labels.extend(set.into_iter());

    let formats = release.formats.clone();

    let display_time = release.date_added
    .with_timezone(&Config::timezone());

    let content = String::from(format!("
    Artist: {}

    Year Released: {}

    Labels: {}

    Formats: {}

    Date Added: {}
    
    Discogs ID: {}",
    release.artist,
    release.year,
    format_vec(labels),
    format_vec(formats),
    display_time,
    release.id,
    ));

    let screen = ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        Dialog::text(content)
            .title(format!("{} - {}", 
            release.artist.clone(), 
            release.title.clone()))
    );
    screen
}

pub fn multiple_results(results: Vec<Release>, from_listen: bool) -> ResizedView<Dialog> {
    let screen = ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        Dialog::around(
            SelectView::<Release>::new()
            .with_all(
                results.clone().into_iter().map(|i| {
                    (format!("{} ({})", i.title.clone(), i.formats[0]), i)
                })
            )
            .on_submit(move |s, item| {
                s.pop_layer();
                if !from_listen {
                    s.add_fullscreen_layer(
                        albuminfo(item)
                    );
                } else {
                    let time_now = utils::get_utc_now();
                    let entry = ListenLogEntry {
                        id: results[0].id,
                        title: results[0].title.clone(),
                        time: time_now,
                    };
                    match update::listenlog(entry) {
                        Ok(()) => {}
                        Err(_) => {}
                    }
                }
            })
        ).title("Multiple results for query")
    );
    screen
}

fn format_vec(list: Vec<String>) -> String {
    let mut formatted_string = String::new();
    if list.len() > 1 {
        for item in &list[0..list.len()-1] {
            formatted_string.push_str(item);
            formatted_string.push_str(", ");
        }
        formatted_string.push_str(&list[list.len()-1]);
    } else {
        formatted_string.push_str(&list[0]);
    }
    formatted_string
}