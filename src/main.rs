use cursive::views::*;
use cursive::traits::*;
use cursive::view::View;

mod app;
mod screens;
mod theme;

use app::App;
use screens::traits::Screen;
use screens::collection::{self, Collection};

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(theme::theme_gen());

    let app = App::new();
    app.load(&mut siv);

    siv.run();
}