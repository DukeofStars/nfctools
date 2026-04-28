use dioxus::prelude::*;
use schemas::Ship;

use crate::{
    components::{
        dropdown_menu::{
            DropdownMenu, DropdownMenuContent, DropdownMenuItem,
            DropdownMenuTrigger,
        },
    },
    fleet_edit::{self, EditableHullParams, get_ln_editable_hull_params},
};

#[component]
pub fn ShipEditor(ship: Signal<Option<Ship>>) -> Element {
    let ship_config_table = use_memo(move || {
        let ship_read = ship.read();
        let Some(ship_r) = ship_read.as_ref() else {
            return rsx! { "No ship selected" };
        };

        if ship_r.hull_type != "Stock/Bulk Hauler" {
            return rsx! { "Ship is not a liner" };
        }

        let hull_params = get_ln_editable_hull_params(ship_r);

        if let Some(hull_params) = hull_params {
            rsx! {
                ShipConfigTable { key: "{ship_r.name}", ship, hull_params }
            }
        } else {
            rsx! { "Ship is not a liner" }
        }
    });

    rsx! {
        div { display: "flex", flex_direction: "column",
            if let Some(ship) = ship.read().as_ref() {
                h1 { "Editing Ship '{ship.name}'" }
            }
            {ship_config_table}
        }
    }
}

/// A small chevron-down icon rendered as inline SVG.
#[component]
fn ChevronDown() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "12",
            height: "12",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "6 9 12 15 18 9" }
        }
    }
}

#[component]
fn DressingDropdown(
    segment: usize,
    slot: usize,
    hull_params: Signal<EditableHullParams>,
) -> Element {
    let segment_key = match segment {
        0 => {
            let segment_key_idx = hull_params.read().bow_type;
            crate::fleet_edit::BULK_BOWS[segment_key_idx]
        }
        1 => {
            let segment_key_idx = hull_params.read().core_type;
            crate::fleet_edit::BULK_CORES[segment_key_idx]
        }
        _ => {
            panic!()
        }
    };
    let dressings = crate::dressings::LN_DRESSINGS
    .get(&(segment_key, slot))
    .unwrap();

    let mut selected_dressing = use_signal(|| {
        match segment {
            0 => hull_params.read().bow_dressings[slot] as usize,
            1 => hull_params.read().core_dressings[slot] as usize,
            _ => panic!()
        }
    });

    // This can occur when a segment changes type but it has dressings set, so we just reset the dressing to 0.
    if *selected_dressing.read() >= dressings.len() {
        *selected_dressing.write() = 0;
    }

    use_effect(move || {
        let idx = selected_dressing.read().clone();
        // This can occur when a segment changes type but it has dressings set, so we just reset the dressing to 0.
        if idx >= dressings.len() {
            *selected_dressing.write() = 0;
            return;
        }
        let idx = idx as u8;
        match segment {
            0 => hull_params.write().bow_dressings[slot] = idx,
            1 => hull_params.write().core_dressings[slot] = idx,
            _ => panic!()
        }
    });

    rsx! {
        DropdownMenu {
            // The dropdown menu trigger is the button that will display the dropdown menu when clicked.
            DropdownMenuTrigger {
                {
                    let selected_dressing_text = dressings[selected_dressing.read().clone()];
                    rsx! { "{selected_dressing_text}" }
                }
                ChevronDown {}
            }
            // The dropdown menu content contains all the items that will be displayed in the dropdown menu.
            DropdownMenuContent {
                for (idx , dressing) in dressings.iter().enumerate() {
                    DropdownMenuItem {
                        // The index of the item, used to determine the order in which items are displayed.
                        index: idx,
                        // The value of the item which will be passed to the on_select callback when the item is selected.
                        value: idx,
                        on_select: move |value: usize| { selected_dressing.set(value) }, // This callback is triggered when the item is selected.,
                        "{dressing}"
                    }
                }
            }
        }
    }
}

#[component]
fn SegmentTypeDropdown(
    segment: usize,
    hull_params: Signal<EditableHullParams>,
) -> Element {
    let mut selected_segment_type = use_signal(|| {
        let segment_type = match segment {
            0 => hull_params.read().bow_type,
            1 => hull_params.read().core_type,
            2 => hull_params.read().stern_type,
            _ => panic!(),
        };
        match segment_type {
            0 => "A",
            1 => "B",
            2 => "C",
            _ => panic!(),
        }
    });
    use_effect(move || {
        let segment_type = match *selected_segment_type.read() {
            "A" => 0,
            "B" => 1,
            "C" => 2,
            _ => panic!(),
        };
        trace!(%segment, %segment_type, "Updating segment type");
        match segment {
            0 => hull_params.write().bow_type = segment_type,
            1 => hull_params.write().core_type = segment_type,
            2 => hull_params.write().stern_type = segment_type,
            _ => panic!()
        }
    });
    rsx! {
        DropdownMenu {
            // The dropdown menu trigger is the button that will display the dropdown menu when clicked.
            DropdownMenuTrigger {
                "{selected_segment_type}"
                ChevronDown {}
            }
            // The dropdown menu content contains all the items that will be displayed in the dropdown menu.
            DropdownMenuContent {
                DropdownMenuItem {
                    // The index of the item, used to determine the order in which items are displayed.
                    index: 0usize,
                    // The value of the item which will be passed to the on_select callback when the item is selected.
                    value: "A",
                    on_select: move |value: &'static str| { selected_segment_type.set(value) }, // This callback is triggered when the item is selected.,
                    "A"
                }
                DropdownMenuItem {
                    // The index of the item, used to determine the order in which items are displayed.
                    index: 1usize,
                    // The value of the item which will be passed to the on_select callback when the item is selected.
                    value: "B",
                    on_select: move |value: &'static str| { selected_segment_type.set(value) }, // This callback is triggered when the item is selected.,
                    "B"
                }
                DropdownMenuItem {
                    // The index of the item, used to determine the order in which items are displayed.
                    index: 2usize,
                    // The value of the item which will be passed to the on_select callback when the item is selected.
                    value: "C",
                    on_select: move |value: &'static str| { selected_segment_type.set(value) }, // This callback is triggered when the item is selected.,
                    "C"
                }
            }
        }
    }
}

#[component]
fn ShipConfigTable(
    ship: Signal<Option<Ship>>,
    hull_params: EditableHullParams,
) -> Element {
    debug!("Creating ShipConfigTable");

    let mut selected_bridge_loc = use_signal(|| hull_params.superstructure_loc);
    let mut hull_params = use_signal(|| hull_params);

    use_effect(move || {
        hull_params.write().superstructure_loc = selected_bridge_loc.read().clone();
    });

    use_effect(move || {
        let hull_params = &*hull_params.read();
        let mut ship_w = ship.write();
        let ship = ship_w.as_mut();
        if let Some(ship) = ship {
            debug!("Updating hull configuration for ship '{}'", ship.name);
            fleet_edit::set_ln_hull_config(ship, hull_params.clone());
        }
    });


    rsx! {
        table { style: "table-layout: fixed;",
            colgroup {
                col { style: "" } // row label
                col { style: "" } // Bow
                col { style: "" } // Core
                col { style: "" } // Stern
            }

            thead {
                tr {
                    th { "" }
                    th { "Bow" }
                    th { "Core" }
                    th { "Stern" }
                }
            }

            tbody {
                tr {
                    td { "Segment Type" }
                    td {
                        SegmentTypeDropdown { segment: 0, hull_params }
                    }
                    td {
                        SegmentTypeDropdown { segment: 1, hull_params }
                    }
                    td {
                        SegmentTypeDropdown { segment: 2, hull_params }
                    }
                }

                tr {
                    td { "Superstructure" }
                    td {
                        input {
                            r#type: "radio",
                            name: "superstructure_loc",
                            checked: selected_bridge_loc() == 0,
                            oninput: move |_| {
                                *selected_bridge_loc.write() = 0;
                            },
                        }
                    }
                    td {
                        input {
                            r#type: "radio",
                            name: "superstructure_loc",
                            checked: selected_bridge_loc() == 1,
                            oninput: move |_| {
                                *selected_bridge_loc.write() = 1;
                            },
                        }
                    }
                    td {
                        input {
                            r#type: "radio",
                            name: "superstructure_loc",
                            checked: selected_bridge_loc() == 2,
                            oninput: move |_| {
                                *selected_bridge_loc.write() = 2;
                            },
                        }
                    }
                }

                for slot in 0..4 {
                    tr {
                        td { "Dressing Slot {slot+1}" }
                        td {
                            DressingDropdown { segment: 0, slot, hull_params }
                        }
                        td {
                            DressingDropdown { segment: 1, slot, hull_params }
                        }
                        td {}
                    }
                }
            }
        }
    }
}
