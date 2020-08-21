use cursive::Cursive;
use cursive::views::*;

use crate::screens::traits::Screen;
use crate::screens::{
    collection::{self, Collection},
    //add other modules as implemented
};
use crate::app::App;

impl App {
    pub fn new() -> Self {
        App {
            user_id: String::from("gay_fucker2000"),
            token: String::from("your mum hella gay")
        }
    }

    pub fn load(&self, s: &mut Cursive) {
        self.add_callbacks(s);
        
        //initialize screen data
        let collect = Collection::new();

        let mut main_screen = ScreensView::new();

        //initialize gui tree
        let collection = main_screen.add_active_screen(collect.build());
        s.add_fullscreen_layer(main_screen);

        //placeholder code
        collection::add_to_list(s, "albumlist", &self.user_id);
        collection::add_to_list(s, "albumlist", &self.token);
        collection::add_to_list(s, "albumlist", "what the fuck");
        collection::add_to_list(s, "folderlist", "main folder");
    }

    fn add_callbacks(&self, s: &mut Cursive) {
        s.add_global_callback('q', |s| s.quit());
        s.add_global_callback(':', |s| {
            s.call_on_name("commandline", |view: &mut EditView| {
                view.enable();
                view.set_content(":");
            });
            s.focus_name("commandline").unwrap();
        });
    }
}