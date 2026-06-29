use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use schemas::Fleet;

use crate::{fleet_data::FleetData, fleet_io::read_fleet};

#[component]
pub fn MergeFleetsDialog(
    fleets: Vec<FleetData>,
    signal: Signal<bool>,
) -> Element {
    let mut out_name = use_signal(|| String::new());
    let mut running = use_signal(|| false);

    rsx! {
        h2 { "Merge Fleets" }
        for fleet in &fleets {
            p { " - {fleet.name}" }
        }
        div { height: "10px" }
        p { "Output Name:" }
        input {
            style: "box-sizing: border-box; width: 100%",
            value: fleets.get(0).unwrap().name.clone(),
            oninput: move |evt| { out_name.set(evt.value()) },
        }

        div { style: "display: flex;",
            button {
                disabled: running(),
                class: "button",
                style: "margin: 10px auto 0; width: 80%; height: 30px;",
                onclick: move |_| {
                    running.set(true);
                    let fleet_datas = fleets.clone();

                    spawn(async move {
                        let Some(file) = AsyncFileDialog::new()
                            .set_title("Save merged fleet")
                            .add_filter("Fleet", &["fleet"])
                            .set_directory(
                                crate::config::APP_CONFIG
                                    .get()
                                    .expect("App configuration not loaded")
                                    .lock()
                                    .unwrap()
                                    .saves_dir
                                    .join("Fleets"),
                            )
                            .save_file()
                            .await else {
                            warn!("Fleet merge aborted; no path selected");
                            return;
                        };
                        info!("Merging {} fleets into {}", fleet_datas.len(), file.path().display());
                        let mut out = Fleet {
                            xmlns_xsd: "".to_string(),
                            xmlns_xsi: "".to_string(),
                            text: None,
                            name: out_name(),
                            description: Some(
                                format!(
                                    "Merged fleets:\n{}",
                                    fleet_datas
                                        .iter()
                                        .map(|f| format!(" - {}", f.name))
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                ),
                            ),
                            version: "".to_string(),
                            total_points: 0,
                            faction_key: "".to_string(),
                            sort_override_order: None,
                            ships: Some(schemas::Ships {
                                text: None,
                                ship: Some(Vec::new()),
                            }),
                            missile_types: Some(schemas::MissileTypes {
                                text: None,
                                missile_template: Some(Vec::new()),
                            }),
                            craft_types: Some(schemas::CraftTypes {
                                text: None,
                                craft_template: Some(Vec::new()),
                            }),
                            mod_dependencies: Some(schemas::ModDependencies {
                                unsigned_long: Some(Vec::new()),
                            }),
                        };
                        for fleet_data in fleet_datas {
                            let fleet = read_fleet(&fleet_data.path).unwrap();
                            debug!("Pulling data from '{}'", fleet_data.name);
                            if out.xmlns_xsd.is_empty() {
                                out.xmlns_xsd = fleet.xmlns_xsd;
                            }
                            if out.xmlns_xsi.is_empty() {
                                out.xmlns_xsi = fleet.xmlns_xsi;
                            }
                            if out.text.is_none() && fleet.text.is_some() {
                                out.text = fleet.text;
                            }
                            if out.version.is_empty() {
                                out.version = fleet.version;
                            } else if out.version != fleet.version {
                                warn!(
                                    "Merging fleets made in different fleet editor versions. This may cause issues"
                                );
                            }
                            out.total_points += fleet.total_points;
                            if out.faction_key.is_empty() {
                                out.faction_key = fleet.faction_key;
                            } else if out.faction_key != fleet.faction_key {
                                error!("Merging fleets of different factions");
                                return;
                            }
                            for ship in fleet
                                .ships
                                .map(|ships| ships.ship)
                                .flatten()
                                .unwrap_or_default()
                            {
                                out.ships.as_mut().unwrap().ship.as_mut().unwrap().push(ship);
                            }
                            for missile in fleet
                                .missile_types
                                .map(|missile_types| missile_types.missile_template)
                                .flatten()
                                .unwrap_or_default()
                            {
                                out.missile_types
                                    .as_mut()
                                    .unwrap()
                                    .missile_template
                                    .as_mut()
                                    .unwrap()
                                    .push(missile);
                            }
                            for craft in fleet
                                .craft_types
                                .map(|craft_types| craft_types.craft_template)
                                .flatten()
                                .unwrap_or_default()
                            {
                                out.craft_types
                                    .as_mut()
                                    .unwrap()
                                    .craft_template
                                    .as_mut()
                                    .unwrap()
                                    .push(craft);
                            }
                            for mod_dep in fleet
                                .mod_dependencies
                                .map(|mod_dep| mod_dep.unsigned_long)
                                .flatten()
                                .unwrap_or_default()
                            {
                                out.mod_dependencies
                                    .as_mut()
                                    .unwrap()
                                    .unsigned_long
                                    .as_mut()
                                    .unwrap()
                                    .push(mod_dep);
                            }
                        }
                        crate::fleet_io::write_fleet(file.path(), &out)
                            .expect("Failed to write fleet file");
                        debug!("Merge complete successfully");
                        running.set(false);
                        signal.set(false);
                    });
                },
                if running() {
                    span { class: "spinner" }
                } else {
                    "Merge {fleets.len()} fleets"
                }
            }
        }
    }
}