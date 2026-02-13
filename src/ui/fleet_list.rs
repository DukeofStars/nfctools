use dioxus::prelude::*;

use crate::{
    config::load_app_config, fleet_data::FleetData, fleet_io::read_fleet,
    load_fleets, spawn_async::spawn_async,
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

    let selected_fleet = use_resource(move || async move {
        if let Some(fleet_data) = selected_fleet_data.as_ref() {
            loading_fleet.set(true);
            let fleet_path = fleet_data.path.clone();
            let fleet = spawn_async(|| read_fleet(fleet_path));
            let fleet = fleet.await;
            loading_fleet.set(false);
            fleet.ok()
        } else {
            None
        }
    });

    rsx! {
        div {
            display: "grid",
            grid_template_columns: "33% 33% 33%",
            overflow: "hidden",
            height: "97vh",
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
            div { "Hello!!!" }
            div {
                display: "flex",
                flex_direction: "column",
                justify_content: "space-between",
                overflow: "hidden",
                h3 { "Hello, World!" }
                div {
                    h3 { "Ships" }
                    div { overflow_y: "scroll", display: "grid",
                        if loading_fleet() {
                            "Loading fleet..."
                        } else {
                            match selected_fleet.read().as_ref() {
                                Some(Some(fleet)) => rsx! {
                                    for ship in fleet
                                        .ships
                                        .iter()
                                        .map(|ships| ships.ship.iter().map(|iter| iter.iter()))
                                        .flatten()
                                        .flatten()
                                    {
                                        {
                                            rsx! {
                                                button { "{ship.name}" }
                                            }
                                        }
                                    }
                                },
                                Some(None) => rsx! {
                                    div { "Error (001)" }
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
