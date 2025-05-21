use floem::{
    prelude::*,
    reactive::SignalRead,
    taffy::{prelude::*, AlignContent, FlexDirection, TrackSizingFunction},
};
use schemas::Fleet;
use tracing::{debug, trace, warn};

use crate::{
    fleet_edit::{
        get_ln_editable_hull_params, set_ln_hull_config, EditableHullParams,
    },
    themes::*,
};

mod dressings;

pub fn fleet_editor(
    selected_ship_idx: RwSignal<usize>,
    selected_fleet: RwSignal<Option<Fleet>>,
) -> impl View {
    let bow_type = create_rw_signal(0);
    let core_type = create_rw_signal(0);
    let stern_type = create_rw_signal(0);

    let dressing_bow = create_rw_signal([0, 0, 0, 0, 0, 0, 0, 0]);
    let dressing_core = create_rw_signal([0, 0, 0, 0, 0, 0, 0, 0]);

    let superstructure_segment = create_rw_signal(0);
    let superstructure_type = create_rw_signal(0);

    v_stack((
        h_stack((
            text("Fleet Editor").style(|s| s.flex_grow(1.0)).style(h1),
            button("Save").style(secondary_button).action(move || {
                selected_fleet.update(|fleet| {
                    if let Some(fleet) = fleet {
                        if let Some(ship) = fleet
                            .ships
                            .as_mut()
                            .map(|ships| ships.ship.as_mut())
                            .flatten()
                            .map(|ships| {
                                ships
                                    .get_mut(selected_ship_idx.get_untracked()
                                        as usize)
                            })
                            .flatten()
                        {
                            if let None = set_ln_hull_config(
                                ship,
                                EditableHullParams {
                                    bow_type: bow_type.get_untracked(),
                                    core_type: core_type.get_untracked(),
                                    stern_type: stern_type.get_untracked(),
                                    bow_dressings: dressing_bow.get_untracked(),
                                    core_dressings: dressing_core
                                        .get_untracked(),
                                    superstructure_loc: superstructure_segment
                                        .get_untracked(),
                                    superstructure_type: superstructure_type
                                        .get_untracked(),
                                },
                            ) {
                                warn!(
                                    "Failed to edit liner hull configuration"
                                );
                            }
                        }
                    }
                });
            }),
        ))
        .style(|s| s.justify_content(AlignContent::SpaceBetween)),
        dyn_view(move || {
            trace!("Re-rendering fleet editor");

            selected_ship_idx.track();

            let binding = selected_fleet.read_untracked();
            let binding = binding.borrow();
            let Some(fleet) = binding.as_ref() else {
                return text("").into_any();
            };
            let ship_idx = selected_ship_idx.get();
            let Some(ship) = fleet
                .ships
                .as_ref()
                .map(|s| s.ship.as_ref())
                .flatten()
                .map(|s| s.get(ship_idx as usize))
                .flatten()
            else {
                return text("").into_any();
            };

            let hull_configs = get_ln_editable_hull_params(ship)
                .unwrap_or_else(|| {
                    debug!("Liner has no hull config, loading default values");
                    EditableHullParams::default()
                });
            bow_type.set(hull_configs.bow_type);
            core_type.set(hull_configs.core_type);
            stern_type.set(hull_configs.stern_type);

            dressing_bow.set(hull_configs.bow_dressings);
            dressing_core.set(hull_configs.core_dressings);

            superstructure_type.set(hull_configs.superstructure_type);
            superstructure_segment.set(hull_configs.superstructure_loc);

            macro_rules! superstructure_selection_checkbox {
                ($segment:tt) => {{
                    radio_button($segment, move || superstructure_segment.get())
                        .style(table_item)
                        .on_update(move |t| superstructure_segment.set(t))
                        .into_any()
                }};
            }
            macro_rules! segment_type_selection_dropbox {
                ($type_signal:expr) => {
                    dropdown::Dropdown::custom(
                        move || ["A", "B", "C"][$type_signal.get()],
                        |s| text(s).style(body).into_any(),
                        ["A", "B", "C"],
                        |s| text(s).style(dropdown_item_view).into_any(),
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
                    .style(table_item)
                    .into_any()
                };
            }
            macro_rules! dressing_selection_row {
                ($slot_idx:expr) => {
                    vec![
                        text(format!("Dressing Slot {}", 0 + 1))
                            .style(table_header)
                            .into_any(),
                        text("").style(table_item).into_any(),
                        dyn_view(move || {
                            dropdown::Dropdown::custom(
                                move || dressing_bow.get()[$slot_idx] as usize,
                                move |dressing| {
                                    text(
                                        dressings::LN_BOW_DRESSINGS
                                            [bow_type.get()][$slot_idx]
                                            [dressing],
                                    )
                                    .style(body)
                                    .into_any()
                                },
                                0..dressings::LN_BOW_DRESSINGS[bow_type.get()]
                                    [$slot_idx]
                                    .len(),
                                move |dressing| {
                                    text(
                                        dressings::LN_BOW_DRESSINGS
                                            [bow_type.get()][$slot_idx]
                                            [dressing],
                                    )
                                    .style(dropdown_item_view)
                                    .into_any()
                                },
                            )
                            .on_accept(move |dressing| {
                                dressing_bow.update(|dressing_bow| {
                                    dressing_bow[$slot_idx] = dressing as u8
                                });
                            })
                            .style(|s| s.width_full())
                            .style(dropdown)
                        })
                        .style(table_item)
                        .into_any(),
                        dyn_view(move || {
                            dropdown::Dropdown::custom(
                                move || dressing_core.get()[$slot_idx] as usize,
                                move |dressing| {
                                    text(
                                        dressings::LN_CORE_DRESSINGS
                                            [core_type.get()][$slot_idx]
                                            [dressing],
                                    )
                                    .style(body)
                                    .into_any()
                                },
                                0..dressings::LN_CORE_DRESSINGS
                                    [core_type.get()][$slot_idx]
                                    .len(),
                                move |dressing| {
                                    text(
                                        dressings::LN_CORE_DRESSINGS
                                            [core_type.get()][$slot_idx]
                                            [dressing],
                                    )
                                    .style(dropdown_item_view)
                                    .into_any()
                                },
                            )
                            .on_accept(move |dressing| {
                                dressing_core.update(|dressing_core| {
                                    dressing_core[$slot_idx] = dressing as u8
                                });
                            })
                            .style(|s| s.width_full())
                            .style(dropdown)
                        })
                        .style(table_item)
                        .into_any(),
                        text("").style(table_item).into_any(),
                    ]
                };
            }

            if ship.hull_type == "Stock/Bulk Hauler" {
                debug!("Loading liner editor panel");
                let mut views_list = vec![
                    // Header Row
                    text("").style(table_item).into_any(),
                    text("").style(table_item).into_any(),
                    text("Bow").style(table_header).into_any(),
                    text("Core").style(table_header).into_any(),
                    text("Stern").style(table_header).into_any(),
                    // Segment Types
                    text("Segment Types").style(table_header).into_any(),
                    text("").style(table_item).into_any(),
                    segment_type_selection_dropbox!(bow_type),
                    segment_type_selection_dropbox!(core_type),
                    segment_type_selection_dropbox!(stern_type),
                    // Superstructure Selection
                    text("Superstructure").style(table_header).into_any(),
                    dyn_view(move || {
                        dropdown::Dropdown::custom(
                            move || superstructure_type.get(),
                            |idx| {
                                text(["A", "B", "C", "D"][idx])
                                    .style(body)
                                    .into_any()
                            },
                            [0, 1, 2, 3],
                            |idx| {
                                text(["A", "B", "C", "D"][idx])
                                    .style(dropdown_item_view)
                                    .into_any()
                            },
                        )
                        .on_accept(move |idx| {
                            superstructure_type.set(idx);
                        })
                        .style(|s| s.width_full())
                        .style(dropdown)
                        .dropdown_style(|dstyle| dstyle)
                    })
                    .style(table_item)
                    .into_any(),
                    superstructure_selection_checkbox!(0),
                    superstructure_selection_checkbox!(1),
                    superstructure_selection_checkbox!(2),
                ];
                views_list.append(&mut dressing_selection_row!(0));
                views_list.append(&mut dressing_selection_row!(1));
                views_list.append(&mut dressing_selection_row!(2));
                views_list.append(&mut dressing_selection_row!(3));
                views_list.append(&mut dressing_selection_row!(4));
                views_list.append(&mut dressing_selection_row!(5));
                views_list.append(&mut dressing_selection_row!(6));
                views_list.append(&mut dressing_selection_row!(7));
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
