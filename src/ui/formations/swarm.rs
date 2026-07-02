use std::collections::HashMap;

use dioxus::prelude::*;

use crate::ui::formations::{viewer3d::Point3, Formation};

#[derive(Debug, Clone)]
pub struct SwarmCompressionConfig {
    pub min_dist: f64,
    pub iterations: usize,
    pub repulsion_strength: f64,
    pub attraction_strength: f64,
    pub damping: f64,
}

pub(super) fn compress_swarm(
    formation: &mut Formation,
    config: SwarmCompressionConfig,
) {
    info!(
        ?config,
        "Compressing formation of {} ships",
        formation.escorts.len() + 1
    );

    let mut points = formation.escorts.clone();
    points.push((formation.lead_ship.clone(), Point3::new(0.0, 0.0, 0.0)) );

    let min_sq = config.min_dist * config.min_dist;

    let mut velocities: HashMap<String, Point3> = HashMap::new();
    for point in &points {
        velocities.insert(point.0.clone(), Point3::new(0.0, 0.0, 0.0));
    }

    for _ in 0..config.iterations {
        let mut forces: HashMap<String, Point3> = HashMap::new();
        for point in &points {
            forces.insert(point.0.clone(), Point3::new(0.0, 0.0, 0.0));
        }

        for i in &points {
            for j in &points {
                if i.0 == j.0 {
                    continue;
                }

                let delta = i.1 - j.1;
                let dist_sq = delta.magnitude_squared();

                if dist_sq < min_sq && dist_sq > 0.0 {
                    let dist = dist_sq.sqrt();
                    let extension = config.min_dist - dist;
                    let dir = delta / dist;

                    let force = Point3::from(
                        dir * extension * config.repulsion_strength,
                    );
                    let force_i = forces.get_mut(&i.0).unwrap();
                    force_i.x += force.x;
                    force_i.y += force.y;
                    force_i.z += force.z;
                }
            }
        }

        for (key, point) in &mut points {
            if *key == formation.lead_ship {
                continue;
            }

            let force = forces.get_mut(key).unwrap();
            force.x -= point.x * config.attraction_strength;
            force.y -= point.y * config.attraction_strength;
            force.z -= point.z * config.attraction_strength;

            let velocity = velocities.get_mut(key).unwrap();
            velocity.x += force.x;
            velocity.y += force.y;
            velocity.z += force.z;

            velocity.x *= config.damping;
            velocity.y *= config.damping;
            velocity.z *= config.damping;

            point.x += velocity.x;
            point.y += velocity.y;
            point.z += velocity.z;
        }
    }

    points.remove(points.len() - 1);
    formation.escorts = points;
}
