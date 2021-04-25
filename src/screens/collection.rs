use cursive::{
    Cursive,
    views::{
        NamedView, 
        LinearLayout,
        Panel,
        ResizedView,
        SelectView,
        ScrollView,
    },
    view::SizeConstraint,
    traits::*
};

use crate::app::{
    database::*,
    Release,
    Folders,
};
use crate::screens::popup;
use crate::APPEARANCE;

#[derive(Debug, Clone)]
pub struct Collection {
    pub folders: Folders
}

impl Collection {
    pub fn new() -> Self {
        Collection {
            folders: query::collection(APPEARANCE.sort_by())
                .unwrap_or(Folders::new())
        }
    }
    pub fn build(&self, folders_width: usize) -> NamedView<LinearLayout> {
        let collection = LinearLayout::horizontal()
            .child(Panel::new(ResizedView::new(
                SizeConstraint::Fixed(folders_width), 
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<Vec<Release>>::new()
                    .with_all(self.folders.contents
                        .clone().into_iter())
                    .on_select(|s, item| {
                        s.call_on_name("albumlist",
                        |view: &mut SelectView<Release>| {
                            view.clear();
                            view.add_all(item.clone().into_iter()
                                .map(|i| {
                                    (i.format(&APPEARANCE.format.as_ref().unwrap()), i)
                                })
                            )   
                        });
                    })
                )
                .with_name("folderlist")))
            )
            .child(Panel::new(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                ScrollView::new(
                    SelectView::<Release>::new()
                    .with_all(self.folders.contents
                        .values().next().unwrap_or(&Vec::new())
                        .clone().into_iter().map(|i| {
                        (i.format(&APPEARANCE.format.as_ref().unwrap()), i)
                    }))
                    .on_submit(|s, item| {
                        s.add_fullscreen_layer(
                            popup::albuminfo(item)
                        );
                    })
                    .with_name("albumlist"))))
                )
            .with_name("main_view");
                    
        collection
    }

    pub fn refresh(&mut self, s: &mut Cursive) {
        //update from database and reload its contents
        //call database method here
        s.call_on_name("folderlist", |view: &mut SelectView<Vec<Release>>| {
            view.clear();
            view.add_all(self.folders.contents().into_iter())
        });
    }
}
