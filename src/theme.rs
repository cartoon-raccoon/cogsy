/*
shamelessly stolen from github.com/NerdyPepper/dijo/
and adapted to my own needs
*/

use cursive::theme::Color::*;
use cursive::theme::BaseColor;
use cursive::theme::PaletteColor::*;
use cursive::theme::{BorderStyle, Palette, Theme};

pub fn palette_gen() -> Palette {
    let mut p = Palette::default();
    p[Background] = TerminalDefault;
    p[Shadow] = TerminalDefault;
    p[View] = TerminalDefault;
    p[Primary] = TerminalDefault;
    p[Secondary] = TerminalDefault;
    p[Tertiary] = TerminalDefault;
    p[TitlePrimary] = TerminalDefault;
    p[Highlight] = TerminalDefault;
    p[HighlightInactive] = TerminalDefault;
    p[HighlightText] = Dark(BaseColor::Yellow);

    return p;
}

pub fn theme_gen() -> Theme {
    let mut t = Theme::default();
    t.shadow = false;
    t.borders = BorderStyle::Simple;
    t.palette = palette_gen();
    return t;
}