use std::fmt::Display;

use floem::{
    prelude::*,
    reactive::SignalRead,
    taffy::{prelude::*, AlignContent, FlexDirection, TrackSizingFunction},
};
use schemas::Ship;
use tracing::{debug, trace};

use crate::themes::*;

mod dressings;

#[derive(PartialEq, Eq, Clone)]
enum SuperstructureSelection {
    Bow,
    Core,
    Stern,
}

#[derive(PartialEq, Eq, Clone)]
enum SuperstructureType {
    A,
    B,
    C,
    D,
}
impl Display for SuperstructureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuperstructureType::A => write!(f, "A"),
            SuperstructureType::B => write!(f, "B"),
            SuperstructureType::C => write!(f, "C"),
            SuperstructureType::D => write!(f, "D"),
        }
    }
}

pub fn fleet_editor(selected_ship: RwSignal<Option<Ship>>) -> impl View {
    v_stack((
        text("Fleet Editor").style(|s| s.flex_grow(1.0)).style(h1),
        dyn_view(move || {
            trace!("Re-rendering fleet editor");

            let binding = selected_ship.read();
            let binding = binding.borrow();
            let Some(ship) = binding.as_ref() else {
                return text("").into_any();
            };

            // TODO: Load fleet data
            let superstructure_selection =
                create_rw_signal(SuperstructureSelection::Bow);
            let superstructure_type = create_rw_signal(SuperstructureType::A);

            macro_rules! superstructure_selection_checkbox {
                ($segment:path) => {{
                    use SuperstructureSelection::*;
                    radio_button($segment, move || {
                        superstructure_selection.get()
                    })
                    .style(grid_item)
                    .on_update(move |t| superstructure_selection.set(t))
                    .into_any()
                }};
            }

            let bow_type = create_rw_signal(0);
            let core_type = create_rw_signal(0);
            let stern_type = create_rw_signal(0);

            macro_rules! segment_type_selection_dropbox {
                ($type_signal:expr) => {
                    dropdown::Dropdown::custom(
                        move || ["A", "B", "C"][$type_signal.get()],
                        |s| text(s).style(body).into_any(),
                        ["A", "B", "C"],
                        |s| text(s).style(body).into_any(),
                    )
                    .on_accept(move |t| {
                        let idx = match t {
                            "A" => 0,
                            "B" => 1,
                            "C" => 2,
                            _ => panic!(),
                        };
                        $type_signal.set(idx);
                    })
                    .style(dropdown)
                    .style(grid_item)
                    .into_any()
                };
            }

            let dressing_bow = create_rw_signal([0, 0, 0, 0, 0]);
            let dressing_core = create_rw_signal([0, 0, 0, 0, 0]);

            macro_rules! dressing_selection_row {
                ($slot_idx:expr) => {
                    vec![
                        text(format!("Dressing Slot {}", $slot_idx + 1))
                            .style(grid_header)
                            .into_any(),
                        text("").style(grid_item).into_any(),
                        dyn_view(move || {
                            dropdown::Dropdown::custom(
                                move || {
                                    &dressings::LN_BOW_DRESSINGS[bow_type.get()]
                                        [$slot_idx]
                                        [dressing_bow.get()[$slot_idx]]
                                },
                                |dressing| {
                                    text(dressing).style(body).into_any()
                                },
                                dressings::LN_BOW_DRESSINGS[bow_type.get()]
                                    [$slot_idx]
                                    .iter(),
                                |dressing| {
                                    text(dressing).style(body).into_any()
                                },
                            )
                            .style(|s| s.width_full())
                            .style(dropdown)
                        })
                        .style(grid_item)
                        .into_any(),
                        dyn_view(move || {
                            dropdown::Dropdown::custom(
                                move || {
                                    &dressings::LN_CORE_DRESSINGS
                                        [core_type.get()][$slot_idx]
                                        [dressing_core.get()[$slot_idx]]
                                },
                                |dressing| {
                                    text(dressing).style(body).into_any()
                                },
                                dressings::LN_CORE_DRESSINGS[core_type.get()]
                                    [$slot_idx]
                                    .iter(),
                                |dressing| {
                                    text(dressing).style(body).into_any()
                                },
                            )
                            .style(|s| s.width_full())
                            .style(dropdown)
                        })
                        .style(grid_item)
                        .into_any(),
                        text("").style(grid_item).into_any(),
                    ]
                };
            }

            if ship.hull_type == "Stock/Bulk Hauler" {
                debug!("Loading liner editor panel");
                let mut views_list = vec![
                    // Header Row
                    text("").style(grid_item).into_any(),
                    text("").style(grid_item).into_any(),
                    text("Bow").style(grid_header).into_any(),
                    text("Core").style(grid_header).into_any(),
                    text("Stern").style(grid_header).into_any(),
                    // Segment Types
                    text("Segment Types").style(grid_header).into_any(),
                    text("").style(grid_item).into_any(),
                    segment_type_selection_dropbox!(bow_type),
                    segment_type_selection_dropbox!(core_type),
                    segment_type_selection_dropbox!(stern_type),
                    // Superstructure Selection
                    text("Superstructure").style(grid_header).into_any(),
                    dyn_view(move || {
                        dropdown::Dropdown::new_rw(
                            // TODO: load actual values
                            superstructure_type,
                            [
                                SuperstructureType::A,
                                SuperstructureType::B,
                                SuperstructureType::C,
                                SuperstructureType::D,
                            ],
                        )
                        .style(|s| s.width_full())
                        .style(dropdown)
                    })
                    .style(grid_item)
                    .into_any(),
                    superstructure_selection_checkbox!(Bow),
                    superstructure_selection_checkbox!(Core),
                    superstructure_selection_checkbox!(Stern),
                ];
                views_list.append(&mut dressing_selection_row!(0));
                views_list.append(&mut dressing_selection_row!(1));
                views_list.append(&mut dressing_selection_row!(2));
                views_list.append(&mut dressing_selection_row!(3));
                views_list.append(&mut dressing_selection_row!(4));
                stack_from_iter(views_list)
                    .style(|s| {
                        grid(s)
                            .grid()
                            .grid_template_columns(vec![
                                TrackSizingFunction::from_length(150.0),
                                TrackSizingFunction::from_length(50.0),
                                TrackSizingFunction::from_length(150.0),
                                TrackSizingFunction::from_length(150.0),
                                TrackSizingFunction::from_length(50.0),
                            ])
                            .grid_template_rows(vec![
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                                TrackSizingFunction::from_flex(1.0),
                            ])
                            .flex_direction(FlexDirection::Row)
                            .height_pct(0.0)
                    })
                    .style(grid)
                    .into_any()
            } else {
                return text("").into_any();
            }
        }),
    ))
    .style(|s| {
        s.align_content(AlignContent::Center)
            .justify_content(AlignContent::Start)
            .padding(2)
            .height_pct(0.0)
    })
}
