use floem::{
    event::EventPropagation,
    prelude::*,
    taffy::{
        prelude::FromLength, AlignContent, AlignItems, Display,
        TrackSizingFunction,
    },
};

use super::WindowState;
use crate::themes::*;

pub fn main_menu(window_state: RwSignal<WindowState>) -> impl IntoView {
    container(
        stack((
            button(text("Fleets")).style(main_menu_button).on_click(
                move |_event| {
                    window_state.set(WindowState::FleetsList {
                        selected_fleet: create_rw_signal(None),
                        selected_fleet_data: create_rw_signal(None),
                    });
                    EventPropagation::Stop
                },
            ),
            button(text("Missiles")).style(main_menu_button),
            button(text("Win Predictor"))
                .style(main_menu_button)
                .on_click(move |_event| {
                    window_state.set(WindowState::WinPredictor);
                    EventPropagation::Stop
                }),
            button(text("Install Manager")).style(main_menu_button),
        ))
        .style(|s| {
            s.display(Display::Grid)
                .align_content(Some(AlignContent::SpaceBetween))
                .justify_content(Some(AlignContent::SpaceBetween))
                .grid_template_columns(vec![
                    TrackSizingFunction::from_length(210.0),
                    TrackSizingFunction::from_length(210.0),
                    TrackSizingFunction::from_length(210.0),
                ])
                .width_pct(50.0)
                .height_pct(50.0)
                .min_height(420.0)
        }),
    )
    .style(|s| {
        s.width_full()
            .height_full()
            .align_content(Some(AlignContent::Center))
            .align_items(AlignItems::Center)
            .justify_content(Some(AlignContent::Center))
    })
}
