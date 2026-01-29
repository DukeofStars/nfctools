use floem::{
    peniko::Color,
    prelude::*,
    views::{h_stack_from_iter, text, Decorators}, // View,
};

const ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
enum CapturePointStatus {
    Neutral,
    Team1,
    Team2,
}

pub fn win_predictor_page() -> impl View {
    let capture_points = Box::new(vec![CapturePointStatus::Neutral; 5]);
    // let fleets_list = im::Vector::from_iter(fleets.into_iter());
    // let fleets_list = create_rw_signal(fleets_list);
    h_stack_from_iter(capture_points.into_iter().enumerate().map(
        move |(i, status)| {
            text(ALPHABET.chars().nth(i).unwrap_or('X')).style(move |s| {
                s.border_radius(100).background(match status {
                    CapturePointStatus::Neutral => Color::LIGHT_GRAY,
                    CapturePointStatus::Team1 => Color::BLUE,
                    CapturePointStatus::Team2 => Color::ORANGE_RED,
                })
            })
        },
    ))
}
