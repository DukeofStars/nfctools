use chumsky::primitive::Container;
use floem::{
    peniko::Color,
    prelude::*,
    reactive::SignalRead,
    views::{h_stack_from_iter, text, Decorators}, // View,
};

const ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Clone, Copy)]
enum CapturePointStatus {
    Neutral,
    Team1,
    Team2,
}

pub fn win_predictor_page() -> impl View {
    let capture_points = vec![CapturePointStatus::Neutral; 5];
    let capture_points = im::Vector::from_iter(capture_points);
    let capture_points = create_rw_signal(capture_points);
    h_stack_from_iter(capture_points.get().iter().enumerate().map(
        |(i, status)| {
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
