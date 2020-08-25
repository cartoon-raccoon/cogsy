use cursive::Cursive;
use cursive::traits::*;
use cursive::views::*;
use cursive::event::{Event, Key};

use crate::screens::{
    collection::{self, Collection},
    wantlist::{self, Wantlist},
    //add other modules as implemented
};
use crate::app::App;

impl App {
    pub fn new() -> Self {
        App {
            user_id: String::from("hello"),
            token: String::from("welcome to cogsy")
        }
    }

    pub fn load(&self, s: &mut Cursive) {
        
        //initialize screen data
        let collected = Collection::new();
        let wants = Wantlist::new();


        //initialize gui tree
        let collection = collected.build();
        // let wantlist = wants.build();

        let message = TextContent::new("Welcome to Cogsy!");
        let messagebox = TextView::new_with_content(message.clone());
        
        let commandline = EditView::new()
            .on_submit( move |s: &mut Cursive, text| {
                //placeholder code until i implement commands
                collection::add_to_list(s, "albumlist", text);
                message.set_content(&format!("Added '{}'", text));
                s.focus_name("albumlist").unwrap();
                s.call_on_name("commandline", |view: &mut EditView| {
                    view.set_content("");
                    view.disable();
                });
            })
            .disabled()
            .with_name("commandline");

        let main_layout = LinearLayout::vertical()
            .child(collection)
            .child(messagebox)
            .child(commandline);
        
        s.add_fullscreen_layer(main_layout);
        add_global_callbacks(s);
        s.add_global_callback('1', move |s| {
            s.add_fullscreen_layer(wants.build());
        });
        //placeholder code
        collection::add_to_list(s, "albumlist", &self.user_id);
        collection::add_to_list(s, "albumlist", &self.token);
        collection::add_to_list(s, "folderlist", "main folder");

    }

}

fn add_global_callbacks(s: &mut Cursive) {
    s.add_global_callback('q', |s| s.quit());
    s.add_global_callback(':', |s| {
        s.call_on_name("commandline", |view: &mut EditView| {
            view.enable();
            view.set_content(":");
        });
        s.focus_name("commandline").unwrap();
    });
    s.add_global_callback(Event::Key(Key::Backspace), |s| {
        if s.screen().len() > 1 {
            s.pop_layer();
        }
    })

    //adding screen change callbacks
}