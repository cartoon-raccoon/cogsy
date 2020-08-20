use cursive::views::*;
use cursive::traits::*;
use cursive::view::View;

mod screens;

use screens::collection;
use screens::collection::Collection;

fn main() {
    let mut siv = cursive::default();

    siv.add_global_callback('q', |s| s.quit());
    //DECLARATIVE PHASE
    let collect = Collection::new();
    let mut main_screen = ScreensView::new();
    let collection = main_screen.add_active_screen(collect.build(&mut siv));
    siv.add_fullscreen_layer(main_screen);
    //DECLARATIVE PHASE END
    collection::add_to_list(&mut siv, "What the fuck");
    siv.run();
}