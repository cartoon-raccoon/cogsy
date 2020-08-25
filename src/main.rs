mod app;
mod screens;
mod theme;

use app::App;

fn main() {
    let mut siv = cursive::default();
    siv.set_theme(theme::theme_gen());

    let app = App::new();
    app.load(&mut siv);

    siv.run();
}