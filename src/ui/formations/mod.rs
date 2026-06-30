use dioxus::prelude::*;
use schemas::Fleet;

mod viewer3d;

use crate::ui::formations::viewer3d::{Canvas3D, Point3, Scene};

#[component]
pub fn FleetFormationViewer(fleet: Resource<Option<Fleet>>) -> Element {
    let mut canvas_size = use_signal(|| (0f64, 0f64));
    
    rsx! {
        h3 { "Formations" }
        if let Some(Some(fleet)) = fleet.read().as_ref() {
            {
                let formations = get_formations(fleet);
                if let Some(formation) = formations.get(0) {
                    let scene = formation_to_scene(formation);
                    rsx! {
                        div {
                            style: "width: 50vw; height: 50vh;",
                            onresize: move |evt: ResizeEvent| {
                                let Ok(size) = evt.get_border_box_size() else { return };
                                canvas_size.set((size.width, size.height));
                            },
                            Canvas3D { size: canvas_size, scene }
                        }
                    }
                } else {
                    rsx! { "This fleet has no formations" }
                }
            }
        } else {

        }
    }
}

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
                ship_form.relative_position.x,
                ship_form.relative_position.y,
                ship_form.relative_position.z,
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
        lines
    }
}