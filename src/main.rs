use cursive::views::*;
use cursive::traits::*;
use cursive::view::View;

mod screens;

use screens::collection::Collection;

fn main() {
    let mut siv = cursive::default();

    siv.add_global_callback('q', |s| s.quit());

    let collect = Collection::new();
    let mut main_screen = ScreensView::new();
    main_screen.add_active_screen(collect.build());
    siv.add_fullscreen_layer(main_screen);
    siv.run();
}