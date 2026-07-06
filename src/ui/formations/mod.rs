use arboard::Clipboard;
use dioxus::prelude::*;
use schemas::{Fleet, InitialFormation, RelativePosition, Ship};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod swarm;
mod viewer3d;

use crate::{
    audio::AUDIO_HANDLER,
    components::dropdown_menu::{
        DropdownMenu, DropdownMenuContent, DropdownMenuItem,
        DropdownMenuTrigger,
    },
    fleet_data::FleetData,
    ui::{
        dialog::{swarm_config::SwarmConfigDialog, DialogWrapper},
        fleet_editor::ChevronDown,
        formations::viewer3d::{Canvas3D, Point3, Scene},
    },
};

#[component]
pub fn FleetFormationViewer(
    fleet: Resource<Option<Fleet>>,
    fleet_data: Signal<Option<FleetData>>,
    selected_ship_idx: Signal<Option<usize>>,
    selected_ship: Signal<Option<Ship>>,
) -> Element {
    let mut canvas_size = use_signal(|| (0f64, 0f64));

    let mut formations = use_signal(|| None);
    // Prevent infinite looping, as formations update -> fleet update -> formations update.
    // Formations update -> fleet update gated behind this signal.
    // Enabled when UI makes changes to formations that must be saved.
    let mut fleet_dirty = use_signal(|| false);
    use_effect(move || {
        fleet_data.read();
        fleet_dirty.set(true);
    });
    use_effect(move || {
        if !fleet_dirty() {
            return;
        }
        formations.set(if let Some(Some(fleet)) = fleet.read().as_ref() {
            Some(get_formations(fleet))
        } else {
            None
        });
        fleet_dirty.set(false);
    });
    let formation_lead_names = use_memo(move || {
        let fleet = fleet.read();
        let formations = formations.read();
        let Some(Some(fleet)) = fleet.as_ref() else {
            return None;
        };
        let Some(formations) = formations.as_ref() else {
            return None;
        };

        Some(
            formations
                .iter()
                .map(move |formation| {
                    let mut lead_ship_name = String::new();
                    for ship in fleet
                        .ships
                        .as_ref()
                        .map(|ships| ships.ship.as_ref())
                        .flatten()
                        .unwrap_or(&Vec::new())
                    {
                        if ship.key == formation.lead_ship {
                            lead_ship_name = ship.name.clone();
                        }
                    }
                    lead_ship_name
                })
                .collect::<Vec<_>>(),
        )
    });
    let mut selected_formation = use_signal(|| 0usize);
    let formation_lead_name = use_memo(move || {
        formation_lead_names
            .read()
            .as_ref()
            .map(|x| x.get(selected_formation()))
            .flatten()
            .map(String::clone)
            .unwrap_or_default()
    });

    let ship_names = use_memo(move || {
        let mut names = Vec::new();

        names.push(formation_lead_name());

        let fleet = fleet.read();
        let Some(Some(fleet)) = fleet.as_ref() else {
            return Vec::new();
        };
        let formations = formations.read();
        let Some(formations) = formations.as_ref() else {
            return Vec::new();
        };
        let Some(formation) = formations.get(selected_formation()) else {
            return Vec::new();
        };
        'escort_loop: for (key, _) in &formation.escorts {
            for ship in fleet
                .ships
                .as_ref()
                .map(|ships| ships.ship.as_ref())
                .flatten()
                .unwrap_or(&Vec::new())
            {
                if ship.key == *key {
                    names.push(ship.name.clone());
                    continue 'escort_loop;
                }
            }
            warn!("Ship key not found in fleet");
            return Vec::new();
        }

        names
    });

    let mut near_point: Signal<Option<usize>> = use_signal(|| None);
    let mut selected_point: Signal<Option<usize>> = use_signal(|| None);

    let mut scene = use_signal(|| None);
    let mut old_form_lead = use_signal(String::new);
    use_effect(move || {
        let formations = formations.read();
        let Some(formations) = formations.as_ref() else {
            return;
        };
        let Some(formation) = formations.get(selected_formation()) else {
            return;
        };
        trace!("Updating scene");
        if formation.lead_ship != old_form_lead() {
            selected_point.set(None);
            near_point.set(None);
            old_form_lead.set(formation.lead_ship.clone());
        }
        scene.set(Some(formation_to_scene(formation)));

        trace!("Updating fleet formation");
        let mut fleet = fleet.write();
        let Some(Some(fleet)) = fleet.as_mut() else {
            return;
        };

        let mut empty_vec = Vec::new();
        for formation in formations {
            for (key, point) in &formation.escorts {
                let mut matched_ship = None;
                for ship in fleet
                    .ships
                    .as_mut()
                    .map(|ships| ships.ship.as_mut())
                    .flatten()
                    .unwrap_or(&mut empty_vec)
                {
                    if &ship.key == key {
                        matched_ship = Some(ship);
                        break;
                    } else if &ship.key == &formation.lead_ship {
                        ship.initial_formation = None;
                    }
                }
                let Some(ship) = matched_ship else {
                    warn!(
                        "Could not find matching ship for formation in fleet"
                    );
                    continue;
                };

                ship.initial_formation = Some(InitialFormation {
                    guide_key: formation.lead_ship.clone(),
                    relative_position: RelativePosition {
                        x: point.x / 10.0,
                        y: point.y / 10.0,
                        z: point.z / 10.0,
                    },
                });
            }
        }

        let fleet_data = fleet_data.read();
        let Some(fleet_data) = fleet_data.as_ref() else {
            return;
        };
        match crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to write fleet file: {:?}", err);
            }
        };
    });

    let selected_ship_point = use_memo(move || {
        if selected_point()? == 0 {
            return Some(Point3::new(0.0, 0.0, 0.0));
        }
        let formations = formations.read();
        let point: &(String, Point3) = formations
            .as_ref()?
            .get(selected_formation())?
            .escorts
            .get(selected_point()? - 1)?;
        Some(point.1.clone())
    });

    let mapped_points = use_signal(Vec::new);

    let mut ctx_x = use_signal(|| 0f64);
    let mut ctx_y = use_signal(|| 0f64);

    let mut update_selected_point = move || {
        let Some(near_point) = near_point() else {
            return;
        };
        selected_point.set(Some(near_point));

        let formations = formations.read();
        let empty_vec = Vec::new();
        let escort_key: &String = if near_point == 0 {
            &formations
                .as_ref()
                .unwrap_or(&empty_vec)
                .get(selected_formation())
                .unwrap()
                .lead_ship
        } else {
            &formations
                .as_ref()
                .unwrap_or(&empty_vec)
                .get(selected_formation())
                .unwrap()
                .escorts
                .get(near_point - 1)
                .unwrap()
                .0
        };
        let mut idx = None;
        let mut ship = None;
        let fleet_read = fleet.read();
        for (ship_idx, s) in fleet_read
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .ships
            .as_ref()
            .map(|ships| ships.ship.as_ref())
            .flatten()
            .unwrap_or(&Vec::new())
            .iter()
            .enumerate()
        {
            if &s.key == escort_key {
                idx = Some(ship_idx);
                ship = Some(s.clone());
            }
        }
        selected_ship_idx.set(idx);
        selected_ship.set(ship);
    };

    let mut show_ctx = use_signal(|| false);

    let mut show_swarm_dialog = use_signal(|| false);

    let mut swarm_config = use_signal(|| None);
    let mut swarm_compression_running = use_signal(|| false);

    use_effect(move || {
        if let Some(config) = swarm_config() {
            swarm_compression_running.set(true);

            let mut formations = formations.write();
            let Some(formations) = formations.as_mut() else {
                return;
            };
            let Some(formation) = formations.get_mut(selected_formation())
            else {
                return;
            };
            swarm::compress_swarm(formation, config);

            swarm_compression_running.set(false);
            show_swarm_dialog.set(false);
            swarm_config.set(None);
        }
    });

    let mut mouse_pos = use_signal(|| (0f64, 0f64));

    rsx! {
        DialogWrapper { signal: show_swarm_dialog,
            SwarmConfigDialog {
                open: show_swarm_dialog,
                config: swarm_config,
                running: swarm_compression_running,
            }
        }

        h3 { "Formations" }

        div { style: "display: flex; flex-direction: row; justify-content: space-between; gap: 10px; align-content: center;",
            div { style: "display: flex; flex-direction: row;",
                p { style: "align-self: center;", "Formation Lead: " }
                DropdownMenu {
                    DropdownMenuTrigger {
                        "{formation_lead_name}"
                        ChevronDown {}
                    }
                    DropdownMenuContent {
                        for (idx , _) in formations.read().as_ref().unwrap_or(&Vec::new()).iter().enumerate() {
                            {
                                let lead_name = formation_lead_names
                                    .read()
                                    .as_ref()
                                    .unwrap_or(&Vec::new())
                                    .get(idx)
                                    .map(String::clone)
                                    .unwrap_or_default();
                                rsx! {
                                    DropdownMenuItem {
                                        index: idx,
                                        value: idx,
                                        on_select: move |value| selected_formation.set(value),
                                        "{lead_name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { style: "display: flex; flex-direction: row; justify-content: end; gap: 3px;",
                button {
                    style: "width: 120px;",
                    class: "button",
                    onclick: move |_| {
                        let Ok(mut clipboard) = Clipboard::new() else {
                            return;
                        };

                        let formations = formations.read();
                        let Some(formations) = formations.as_ref() else {
                            return;
                        };
                        let Some(formation) = formations.get(selected_formation())
                        else {
                            return;
                        };
                        let template = formation.to_template();
                        let Ok(s) = crate::export::export_formation(&template) else {
                            warn!("Failed to export formation");
                            return;
                        };
                        info!("Exported formation: '{}'", & s);
                        clipboard.set_text(s).unwrap();
                    },
                    "Copy to clipboard"
                }
                button {
                    style: "width: 140px;",
                    class: "button",
                    onclick: move |_| {
                        let Ok(mut clipboard) = Clipboard::new() else {
                            return;
                        };

                        let mut formations = formations.write();
                        let Some(formations) = formations.as_mut() else {
                            return;
                        };
                        let Some(formation) = formations.get_mut(selected_formation())
                        else {
                            return;
                        };

                        let Ok(clipboard_text) = clipboard.get_text() else {
                            return;
                        };
                        let Ok(new_form) = crate::export::import_formation(&clipboard_text) else {
                            return;
                        };

                        info!("Importing formation: {:?}", & new_form);

                        for ((_, old_escort), new_escort) in formation
                            .escorts
                            .iter_mut()
                            .zip(new_form.escorts.into_iter())
                        {
                            *old_escort = new_escort.into();
                        }
                    },
                    "Paste from clipboard"
                }
            }
        }

        div {
            style: "width: 50vw; height: 50vh; position: relative;",
            onresize: move |evt: ResizeEvent| {
                let Ok(size) = evt.get_border_box_size() else { return };
                canvas_size.set((size.width, size.height));
            },
            oncontextmenu: move |evt| {
                evt.prevent_default();
                let coords = evt.client_coordinates();
                ctx_x.set(coords.x);
                ctx_y.set(coords.y);
                update_selected_point();
                show_ctx.set(true);
            },
            onclick: move |_| {
                show_ctx.set(false);
                update_selected_point()
            },
            onmousemove: move |evt: MouseEvent| {
                mouse_pos.set((evt.client_coordinates().x, evt.client_coordinates().y));

                const SNAPPING_DIST: f64 = 20.0;

                let coords = evt.element_coordinates();
                let mouse_x = coords.x;
                let mouse_y = coords.y;

                let mut min_dist_sq = f64::MAX;
                let mut point_idx = 0;
                for (idx, (x, y)) in mapped_points.read().iter().enumerate() {
                    let dist_sq = (mouse_x - x) * (mouse_x - x)
                        + (mouse_y - y) * (mouse_y - y);
                    if dist_sq < min_dist_sq {
                        min_dist_sq = dist_sq;
                        point_idx = idx;
                    }
                }
                scene
                    .write()
                    .as_mut()
                    .map(|scene| {
                        if min_dist_sq <= SNAPPING_DIST * SNAPPING_DIST {
                            scene.highlight_points.clear();
                            scene.highlight_points.push((point_idx, "#a0a0a0".to_string()));
                            near_point.set(Some(point_idx));
                        } else {
                            scene.highlight_points.clear();
                            near_point.set(None);
                        }
                        if let Some(selected_point) = selected_point() {
                            scene.highlight_points.push((selected_point, "#f0f0f0".to_string()));
                        }
                    });
            },
            if show_ctx() {
                div {
                    style: "position: fixed; top: {ctx_y}px; left: {ctx_x}px; tab-index: 0; display: flex; flex-direction: column; z-index: 100;",
                    class: "context-container",
                    onfocusout: move |_| show_ctx.set(false),
                    button {
                        class: "context-button",
                        onmouseenter: move |_| AUDIO_HANDLER.play_hover_sound(),
                        onclick: move |_| {
                            show_ctx.set(false);
                            show_swarm_dialog.set(true);
                        },
                        "Compress swarm"
                    }
                    if let Some(selected_point) = selected_point() {
                        if selected_point != 0 {
                            button {
                                class: "context-button",
                                onmouseenter: move |_| AUDIO_HANDLER.play_hover_sound(),
                                onclick: move |_| {
                                    show_ctx.set(false);
                                    let mut formations = formations.write();
                                    let Some(formations) = formations.as_mut() else {
                                        return;
                                    };
                                    let Some(formation) = formations.get_mut(selected_formation()) else {
                                        return;
                                    };
                                    change_leader(formation, selected_point - 1);
                                },
                                "Make leader"
                            }
                        }
                    }
                }
            }
            for (i , (x , y)) in mapped_points.read().iter().enumerate() {
                p { style: "position: absolute; left: {x}px; top: {y+2.0}px; font-size: 10px; anchor: top;",
                    "{ship_names.get(i).map(|s| s.clone()).unwrap_or_default()}"
                }
            }
            div {

                if scene.read().is_some() {
                    Canvas3D {
                        size: canvas_size,
                        mouse_pos,
                        scene,
                        mapped_points,
                    }
                }
            }
        }

        if let Some(selected_point) = selected_point() {
            if selected_point == 0 {
                "Ship is leader"
            } else {
                div { style: "display: grid; grid-template-columns: 40% 60%; width: 50%;",
                    "Relative X:"
                    input {
                        value: "{selected_ship_point().map(|point| point.x).unwrap_or_default()}",
                        onchange: move |evt| {
                            let mut formations = formations.write();
                            let Some(formations) = formations.as_mut() else { return };
                            let Some(formation): Option<&mut Formation> = formations
                                .get_mut(selected_formation()) else { return };
                            let entry: Option<&mut (String, Point3)> = formation
                                .escorts
                                .get_mut(selected_point - 1);
                            if let Some((_, point)) = entry {
                                let Ok(parsed) = f64::from_str(&evt.value()) else {
                                    warn!("Invalid X coordinate");
                                    return;
                                };
                                point.x = parsed;
                            }
                        },
                    }
                    "Relative Y:"
                    input {
                        value: "{selected_ship_point().map(|point| point.y).unwrap_or_default()}",
                        onchange: move |evt| {
                            let mut formations = formations.write();
                            let Some(formations) = formations.as_mut() else { return };
                            let Some(formation): Option<&mut Formation> = formations
                                .get_mut(selected_formation()) else { return };
                            let entry: Option<&mut (String, Point3)> = formation
                                .escorts
                                .get_mut(selected_point - 1);
                            if let Some((_, point)) = entry {
                                let Ok(parsed) = f64::from_str(&evt.value()) else {
                                    warn!("Invalid Y coordinate");
                                    return;
                                };
                                point.y = parsed;
                            }
                        },
                    }
                    "Relative Z:"
                    input {
                        value: "{selected_ship_point().map(|point| point.z).unwrap_or_default()}",
                        onchange: move |evt| {
                            let mut formations = formations.write();
                            let Some(formations) = formations.as_mut() else { return };
                            let Some(formation): Option<&mut Formation> = formations
                                .get_mut(selected_formation()) else { return };
                            let entry: Option<&mut (String, Point3)> = formation
                                .escorts
                                .get_mut(selected_point - 1);
                            if let Some((_, point)) = entry {
                                let Ok(parsed) = f64::from_str(&evt.value()) else {
                                    warn!("Invalid Z coordinate");
                                    return;
                                };
                                point.z = parsed;
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Formation {
    lead_ship: String,
    escorts: Vec<(String, Point3)>,
}
impl Formation {
    fn to_template(&self) -> FormationTemplate {
        let mut points = Vec::new();
        for (_, point) in &self.escorts {
            points.push(point.clone().into());
        }
        FormationTemplate { escorts: points }
    }
}

fn get_formations(fleet: &Fleet) -> Vec<Formation> {
    let mut formations: Vec<Formation> = Vec::new();

    let ships = match fleet
        .ships
        .as_ref()
        .map(|ships| ships.ship.as_ref())
        .flatten()
    {
        Some(ships) => ships,
        None => &Vec::new(),
    };
    for ship in ships {
        if let Some(ship_form) = &ship.initial_formation {
            let mut found_form = false;
            let point = Point3::new(
                ship_form.relative_position.x * 10.0,
                ship_form.relative_position.y * 10.0,
                ship_form.relative_position.z * 10.0,
            );
            for formation in &mut formations {
                if ship_form.guide_key == formation.lead_ship {
                    formation.escorts.push((ship.key.clone(), point));
                    found_form = true;
                    break;
                }
            }
            if !found_form {
                formations.push(Formation {
                    lead_ship: ship_form.guide_key.clone(),
                    escorts: vec![(ship.key.clone(), point)],
                })
            }
        }
    }

    formations
}

fn formation_to_scene(form: &Formation) -> Scene {
    let mut points = Vec::new();
    points.push(Point3::new(0.0, 0.0, 0.0));

    let mut lines = Vec::new();

    for escort in &form.escorts {
        points.push(escort.1.clone());
        lines.push((0, points.len() - 1));
    }

    Scene {
        points,
        lines,
        highlight_points: Vec::new(),
    }
}

fn change_leader(formation: &mut Formation, new_lead: usize) {
    let (lead_ship, new_centre) = formation.escorts.remove(new_lead);
    formation
        .escorts
        .push((formation.lead_ship.clone(), Point3::new(0.0, 0.0, 0.0)));
    formation.lead_ship = lead_ship;

    for (_key, point) in &mut formation.escorts {
        point.x -= new_centre.x;
        point.y -= new_centre.y;
        point.z -= new_centre.z;
    }
}

// Can be taken from one fleet and applied to other fleets, therefore it is simply a formation with the ship keys stripped.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FormationTemplate {
    pub escorts: Vec<Point3Serde>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy, PartialEq)]
pub struct Point3Serde {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl From<Point3> for Point3Serde {
    fn from(value: Point3) -> Self {
        Point3Serde {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
impl Into<Point3> for Point3Serde {
    fn into(self) -> Point3 {
        Point3::new(self.x, self.y, self.z)
    }
}
