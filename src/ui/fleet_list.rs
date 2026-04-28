use dioxus::prelude::*;
use schemas::Ship;

use crate::{
    config::load_app_config, fleet_data::FleetData, fleet_io::read_fleet,
    load_fleets, spawn_async::spawn_async, ui::fleet_editor::ShipEditor,
};

#[component]
pub fn FleetList() -> Element {
    let fleets = use_resource(async move || {
        // Load app configuration first
        spawn_async(load_app_config).await.unwrap();
        // Then load fleets (load_fleets requires APP_CONFIG to be set)
        spawn_async(load_fleets::load_fleets).await
    });

    let mut selected_fleet_data = use_signal(|| None::<FleetData>);

    let mut loading_fleet = use_signal(|| false);

    let mut selected_ship = use_signal(|| None::<Ship>);
    let mut selected_ship_idx = use_signal(|| None::<usize>);

    let mut description = use_signal(|| String::new());

    let mut selected_fleet = use_resource(move || async move {
        if let Some(fleet_data) = selected_fleet_data.as_ref() {
            loading_fleet.set(true);
            selected_ship.set(None);
            selected_ship_idx.set(None);
            let fleet_path = fleet_data.path.clone();
            let fleet = spawn_async(|| read_fleet(fleet_path));
            let fleet = fleet.await;
            if let Some(desc) = fleet.as_ref().ok().and_then(|f| f.description.as_ref()) {
                *description.write() = desc.clone();
            }
            loading_fleet.set(false);
            fleet.ok()
        } else {
            None
        }
    });

    use_effect(move || {
        let ship = selected_ship.read();
        if let Some(ship) = ship.as_ref() {
            let fleet_data_r = selected_fleet_data.read();
            let fleet_data = fleet_data_r.as_ref().expect("Ship updated without any fleet being selected");
            let mut fleet_w = selected_fleet.write();
            let fleet = fleet_w.as_mut().unwrap().as_mut().expect("Ship updated without any fleet being selected");
            let ship_idx = selected_ship_idx.read().expect("Ship updated without any ship idx being set");
            fleet.ships.as_mut().unwrap().ship.as_mut().unwrap()[ship_idx] = ship.clone();
            crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet).expect("Failed to write fleet");
        }
    });

    use_effect(move || {
        let desc = description();
        let fleet_data_r = selected_fleet_data.read();
        let Some(fleet_data) = fleet_data_r.as_ref() else {
            return;
        };
        let mut fleet_w = selected_fleet.write();
        let Some(fleet) = fleet_w.as_mut().unwrap().as_mut() else {
            return;
        };
        fleet.description = Some(desc);
        crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet).expect("Failed to write fleet");
    });

    rsx! {
        div {
            display: "grid",
            grid_template_columns: "25% 50% 25%",
            overflow: "hidden",
            // height: "97vh",
            // Fleets List
            div {
                display: "flex",
                flex_direction: "column",
                min_height: 0,
                flex: 1,
                h2 { margin: 0, padding: 0, "Fleets" }
                div { overflow_y: "scroll", display: "grid",
                    // grid_template_columns: "",
                    match fleets.read().as_ref() {
                        Some(Ok(fleets)) => rsx! {
                            for fleet in fleets {
                                {
                                    let fleet = fleet.clone();
                                    rsx! {
                                        button {
                                            onclick: move |_| {
                                                println!("Selected fleet {}", fleet.name);
                                                loading_fleet.set(true);
                                                selected_fleet_data.set(Some(fleet.clone()));
                                            },
                                            "{fleet.name}"
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => {
                            warn!("Failed to load fleets: {}", err);
                            rsx! {
                                div { "Failed to load fleets" }
                            }
                        }
                        None => rsx! {
                            div { "Loading fleets…" }
                        },
                    }
                }
            }
            // Fleet editor (middle)
            ShipEditor { ship: selected_ship }
            div {
                display: "flex",
                flex_direction: "column",
                justify_content: "start",
                overflow: "hidden",
                {
                    match selected_fleet.read().as_ref() {
                        Some(Some(fleet)) => rsx! {
                            h3 { "{fleet.name}" }
                            textarea {
                                height: "200px",
                                value: description(),
                                oninput: move |evt| { description.set(evt.value()) },
                            }
                        },
                        _ => rsx! { "no fleet selected" },
                    }
                }
                div {
                    h3 { "Ships" }
                    div { overflow_y: "scroll", display: "grid",
                        if loading_fleet() {
                            "Loading fleet..."
                        } else {
                            match selected_fleet.read().as_ref() {
                                Some(Some(fleet)) => rsx! {
                                    for (idx , ship) in fleet
                                        .ships
                                        .iter()
                                        .map(|ships| ships.ship.iter().map(|iter| iter.iter()))
                                        .flatten()
                                        .flatten()
                                        .enumerate()
                                    {
                                        {

                                            let ship = ship.clone();
                                            rsx! {
                                                button {
                                                    onclick: move |_| {
                                                        trace!("Selecting ship {}", ship.name);
                                                        selected_ship.set(Some(ship.clone()));
                                                        selected_ship_idx.set(Some(idx.clone()));
                                                    },
                                                    "{ship.name}"
                                                }
                                            }
                                        }
                                    }
                                },
                                Some(None) => rsx! {
                                    div {}
                                },
                                None => rsx! {
                                    div { "Error (002)" }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
