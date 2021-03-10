use std::collections::HashSet;

use cursive::{
    views::{
        ResizedView, 
        Dialog,
        SelectView,
        TextView,
    },
    view::SizeConstraint
};

use crate::utils;
use crate::{CONFIG, APPEARANCE};
use crate::app::{
    Release,
    ListenLogEntry,
    database::{update, query},
};

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
    .with_timezone(&CONFIG.timezone());

    let content = String::from(format!("
    Artist: {}

    Year Released: {}

    Labels: {}

    Formats: {}

    Date Added: {}
    
    Discogs ID: {}",
    release.artist,
    release.year,
    labels.join(", "),
    formats.join(", "),
    display_time.format("%A %d %m %Y %R"),
    release.id,
    ));

    let title = release.title.clone();
    let title2 = title.clone();
    let artist = release.artist.clone();
    let id = release.id;

    let screen = ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        Dialog::text(content)
            .title(format!("{} - {}", 
            artist, title))
            .button("Ok", move |s| {
                s.pop_layer();
            })
            .button("History", move |s| {
                match query::listenlog_by_title(&title2) {
                    Ok(log) => {
                        s.add_fullscreen_layer(
                            log.build_history_title()
                        );
                    }
                    Err(e) => {
                        s.call_on_name("messagebox", |view: &mut TextView| {
                            view.set_content(e.to_string());
                            view.set_style(APPEARANCE.error_col());
                        });
                    }
                }
            })
            .button("Listen", move |s| {
                let entry = ListenLogEntry {
                    id: id,
                    title: &title,
                    time: utils::get_utc_now(),
                };

                match update::listenlog(entry) {
                    Ok(()) => {
                        s.call_on_name("messagebox", |view: &mut TextView| {
                            view.set_content(format!("Listening to `{}` by {}", title, artist))
                        });
                        s.pop_layer();
                    }
                    Err(e) => {
                        s.call_on_name("messagebox", |view: &mut TextView| {
                            view.set_content(format!("Error: {}", e))
                        });
                        s.pop_layer();
                    }
                }
            })
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
                        title: &results[0].title,
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

pub fn format_vec(list: &Vec<String>) -> String {
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