use std::str::FromStr;
use dioxus::prelude::*;
use schemas::{Fleet, InitialFormation, RelativePosition, Ship};

mod viewer3d;

use crate::{components::dropdown_menu::{DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger}, ui::{fleet_editor::ChevronDown, formations::viewer3d::{Canvas3D, Point3, Scene}}};

#[component]
pub fn FleetFormationViewer(fleet: Resource<Option<Fleet>>, selected_ship_idx: Signal<Option<usize>>, selected_ship: Signal<Option<Ship>>) -> Element {
    let mut canvas_size = use_signal(|| (0f64, 0f64));
    
    let mut formations = use_memo(move || {
        if let Some(Some(fleet)) = fleet.read().as_ref() {
            Some(get_formations(fleet))
        } else {
            None
        }
    });
    let formation_lead_names = use_memo(move || {
        let fleet = fleet.read();
        let formations = formations.read();
        let Some(Some(fleet)) = fleet.as_ref() else { return None };
        let Some(formations) = formations.as_ref() else { return None };

        Some(formations.iter().map(move |formation| {
            let mut lead_ship_name = String::new();
            for ship in fleet.ships.as_ref().map(|ships| ships.ship.as_ref()).flatten().unwrap_or(&Vec::new()) {
                if ship.key == formation.lead_ship {
                    lead_ship_name = ship.name.clone();
                }
            }
            lead_ship_name
        }).collect::<Vec<_>>())
    });
    let mut selected_formation = use_signal(|| 0usize);
    let formation_lead_name = use_memo(move || {
        formation_lead_names.read().as_ref().map(|x| x.get(selected_formation())).flatten().map(String::clone).unwrap_or_default()
    });
    
    let mut near_point = use_signal(|| None);
    let mut selected_point = use_signal(|| None);
    
    let mut scene = use_signal(|| None);
    let mut old_form_lead = use_signal(String::new);
    use_effect(move || {
        let formations = formations.read();
        let Some(formations) = formations.as_ref() else { return };
        let Some(formation) = formations.get(selected_formation()) else { return };
        trace!("Updating scene");
        if formation.lead_ship != old_form_lead() {
            selected_point.set(None);
            near_point.set(None);
            old_form_lead.set(formation.lead_ship.clone());
        }
        scene.set(Some(formation_to_scene(formation)));

        trace!("Updating fleet formation");
        let mut fleet = fleet.write();
        let Some(Some(fleet)) = fleet.as_mut() else { return; };
        
        let mut empty_vec = Vec::new();
        for (key, point) in &formation.escorts {
            let mut matched_ship = None;
            for ship in fleet.ships.as_mut().map(|ships| ships.ship.as_mut()).flatten().unwrap_or(&mut empty_vec) {
                if &ship.key == key {
                    matched_ship = Some(ship);
                }
            }
            let Some(ship) = matched_ship else { warn!("Could not find matching ship for formation in fleet"); continue };

            ship.initial_formation = Some(InitialFormation { guide_key: formation.lead_ship.clone(), relative_position: RelativePosition { x: point.x / 10.0, y: point.y / 10.0, z: point.z / 10.0 } });
        }
    });

    let mapped_points = use_signal(|| Vec::new());

    let selected_ship_point = use_memo(move || {
        if selected_point()? == 0 {
            return Some(Point3::new(0.0, 0.0, 0.0));
        }
        let formations = formations.read();
        let point: &(String, Point3) = formations.as_ref()?.get(selected_formation())?.escorts.get(selected_point()? - 1)?;
        Some(point.1.clone())
    });

    rsx! {
        h3 { "Formations" }

        div { style: "display: flex; flex-direction: row; justify-content: start; gap: 10px; align-content: center;",
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

        div {
            style: "width: 50vw; height: 50vh;",
            onresize: move |evt: ResizeEvent| {
                let Ok(size) = evt.get_border_box_size() else { return };
                canvas_size.set((size.width, size.height));
            },
            div {
                onmousemove: move |evt: MouseEvent| {
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
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                },
                onclick: {
                    move |_| {
                        let Some(near_point) = near_point() else { return };
                        selected_point.set(Some(near_point));

                        let formations = formations.read();
                        let empty_vec = Vec::new();
                        let escort_key = if near_point == 0 {
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
                    }
                },
                if scene.read().is_some() {
                    Canvas3D { size: canvas_size, scene, mapped_points }
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
    escorts: Vec<(String, Point3)>
}

fn get_formations(fleet: &Fleet) -> Vec<Formation> {
    let mut formations: Vec<Formation> = Vec::new();

    let ships = match fleet.ships.as_ref().map(|ships| ships.ship.as_ref()).flatten() {
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
                    escorts: vec![(ship.key.clone(), point)]
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