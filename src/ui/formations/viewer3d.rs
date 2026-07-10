use dioxus::{html::geometry::WheelDelta, prelude::*};
use nalgebra::{Isometry3, Perspective3, Vector3};
use serde::Serialize;

use crate::util::spawn_async::spawn_async;

pub type Point3 = nalgebra::Point3<f64>;

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub points: Vec<Point3>,
    pub lines: Vec<(usize, usize)>,
    pub highlight_points: Vec<(usize, String)>,
}

// A scene that has been mapped from 3d world space to 2d space, and is ready for rendering
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MappedScene {
    pub points: Vec<(f64, f64)>,
    lines: Vec<Line>,
    width: f64,
    height: f64,
    pitch: f64,
    yaw: f64,
    camera_distance: f64,
    highlight_points: Vec<(usize, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct Line {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: String,
    width: f64,
}

const NEAR: f64 = 0.1;
const FAR: f64 = 500.0;

fn build_view_projection(
    camera_pos: &Point3,
    width: f64,
    height: f64,
) -> (Isometry3<f64>, Perspective3<f64>) {
    let target = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::y();
    let view: Isometry3<f64> = Isometry3::look_at_rh(camera_pos, &target, &up);

    let fov = 3.14159 / 4.0; // 90deg (nebulous max FOV)
    let aspect_ratio = width / height;
    let projection = Perspective3::new(aspect_ratio, fov, NEAR, FAR);

    (view, projection)
}

fn project_view_point(
    view_point: &Point3,
    projection: &Perspective3<f64>,
    width: f64,
    height: f64,
) -> (f64, f64) {
    let clip_point = projection.project_point(view_point);
    let screen_x = (clip_point.x + 1.0) * 0.5 * width;
    // flip y because screen space usually has y increasing downward
    let screen_y = (1.0 - clip_point.y) * 0.5 * height;
    (screen_x, screen_y)
}

/// Clip a line segment (already in view space, camera looking down -z) against
/// the near plane z = -near. Returns None if the whole segment is behind it.
fn clip_segment_near(
    a: Point3,
    b: Point3,
    near: f64,
) -> Option<(Point3, Point3)> {
    let a_visible = a.z < -near;
    let b_visible = b.z < -near;

    if !a_visible && !b_visible {
        return None;
    }
    if a_visible && b_visible {
        return Some((a, b));
    }

    // Exactly one endpoint is behind the near plane: find where the segment
    // crosses z = -near and trim it there instead of projecting the
    // out-of-frustum endpoint.
    let t = (-near - a.z) / (b.z - a.z);
    let clipped =
        Point3::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y), -near);

    if a_visible {
        Some((a, clipped))
    } else {
        Some((clipped, b))
    }
}

fn project_points(
    points: &[Point3],
    view: &Isometry3<f64>,
    projection: &Perspective3<f64>,
    width: f64,
    height: f64,
) -> Vec<(f64, f64)> {
    points
        .iter()
        .filter_map(|p| {
            let vp = view.transform_point(p);
            if vp.z < -NEAR {
                let (x, y) = project_view_point(&vp, projection, width, height);
                Some((x, y))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// Projects a set of edges (clipping each against the near plane first) and
/// returns a JS array literal of [x1, y1, x2, y2] segments.
fn project_lines(
    points: &[Point3],
    edges: &[(usize, usize)],
    view: &Isometry3<f64>,
    projection: &Perspective3<f64>,
    width: f64,
    height: f64,
) -> Vec<(f64, f64, f64, f64)> {
    let view_points: Vec<Point3> =
        points.iter().map(|p| view.transform_point(p)).collect();

    edges
        .iter()
        .filter_map(|&(a, b)| {
            let va = view_points[a];
            let vb = view_points[b];
            clip_segment_near(va, vb, NEAR).map(|(ca, cb)| {
                let (x1, y1) =
                    project_view_point(&ca, projection, width, height);
                let (x2, y2) =
                    project_view_point(&cb, projection, width, height);
                (x1, y1, x2, y2)
            })
        })
        .collect()
}

const CONTAINER_ID: &str = "canvas-container";
const CANVAS_ID: &str = "scene-canvas";
const CAMERA_DISTANCE: f64 = 1000.0;

fn map_scene(
    scene: &Scene,
    width: f64,
    height: f64,
    pitch: f64,
    yaw: f64,
    camera_distance: f64,
) -> MappedScene {
    // Px = distance * sin(yaw) * cos(pitch)
    // Py = distance * sin(pitch)
    // Pz = distance * cos(yaw) * cos(pitch)
    let camera_x = camera_distance * yaw.sin() * pitch.cos();
    let camera_y = camera_distance * pitch.sin();
    let camera_z = camera_distance * yaw.cos() * pitch.cos();
    let camera_pos = Point3::new(camera_x, camera_y, camera_z);

    let (view, projection) = build_view_projection(&camera_pos, width, height);

    let points =
        project_points(&scene.points, &view, &projection, width, height);

    let lines = project_lines(
        &scene.points,
        &scene.lines,
        &view,
        &projection,
        width,
        height,
    )
    .into_iter()
    .map(|(x1, y1, x2, y2)| Line {
        x1,
        y1,
        x2,
        y2,
        color: String::from("blue"),
        width: 2.0,
    });

    let grid_width: i32 = 10;
    let step = 100.0;

    let mut grid_points = vec![];
    let mut grid_lines = vec![];
    for i in -grid_width..=grid_width {
        for j in -grid_width..=grid_width {
            let x = i as f64 * step;
            let z = j as f64 * step;
            grid_points.push(Point3::new(x, 0.0, z));

            if i < grid_width {
                grid_lines.push((
                    grid_points.len() - 1,
                    grid_points.len() + 2 * grid_width as usize,
                ));
            }
            if j < grid_width {
                grid_lines.push((grid_points.len() - 1, grid_points.len()));
            }
        }
    }

    let grid_lines = project_lines(
        &grid_points,
        &grid_lines,
        &view,
        &projection,
        width,
        height,
    )
    .into_iter()
    .map(|(x1, y1, x2, y2)| Line {
        x1,
        y1,
        x2,
        y2,
        color: String::from("grey"),
        width: 1.0,
    });

    let lines = lines.chain(grid_lines).collect();

    MappedScene {
        points,
        lines,
        width,
        height,
        pitch,
        yaw,
        camera_distance,
        highlight_points: scene.highlight_points.clone(),
    }
}

const MAX_PITCH: f64 = std::f64::consts::FRAC_PI_2 - 0.001;
const MIN_CAM_DIST: f64 = 100.0;

#[component]
pub fn Canvas3D(
    size: Signal<(f64, f64)>,
    scene: Signal<Option<Scene>>,
    mapped_scene: Signal<Option<MappedScene>>,
    children: Element,
    mouse_pos: Signal<(f64, f64)>,
    dragging: Signal<bool>,
) -> Element {
    let mut pitch = use_signal(|| 1.0_f64);
    let mut yaw = use_signal(|| 0.0_f64);
    let mut camera_distance = use_signal(|| CAMERA_DISTANCE);

    let mut last_pos = use_signal(|| (0.0_f64, 0.0_f64));
    const DRAG_SENSITIVITY: f64 = 0.01;

    use_effect(move || {
        mouse_pos.read();

        if dragging() {
            let c = mouse_pos();
            let (last_x, last_y) = last_pos.peek().clone();
            let dx = c.0 - last_x;
            let dy = c.1 - last_y;
            yaw -= dx * DRAG_SENSITIVITY;
            let old_pitch = pitch.peek().clone();
            pitch.set(
                (old_pitch + dy * DRAG_SENSITIVITY)
                    .clamp(-MAX_PITCH, MAX_PITCH),
            );
            last_pos.set((c.0, c.1));
        }
    });

    use_effect(move || {
        trace!("Mapping scene");

        size.read();
        scene.read();
        pitch.read();
        yaw.read();
        camera_distance.read();

        spawn(async move {
            let (width, height) = size();
            let scene = scene().unwrap();
            let pitch = pitch();
            let yaw = yaw();
            let camera_distance = camera_distance();
            let new_mapped_scene = spawn_async(move || {
                map_scene(&scene, width, height, pitch, yaw, camera_distance)
            })
            .await;
            trace!("Scene mapped");
            document::eval(&format!(
                "{}\nrender({})",
                include_str!("renderer.js"),
                serde_json::to_string(&new_mapped_scene).unwrap()
            ));
            mapped_scene.set(Some(new_mapped_scene));
        });
    });

    rsx! {
        div {
            id: CONTAINER_ID,
            onwheel: move |evt: WheelEvent| {
                let vec_y;
                let scroll_multiplier;

                match evt.delta() {
                    WheelDelta::Pixels(vec) => {
                        debug!("Scrolling by pixels");
                        vec_y = vec.y;
                        scroll_multiplier = 0.5;
                    }
                    WheelDelta::Lines(vec) => {
                        debug!("Scrolling by lines");
                        vec_y = vec.y;
                        scroll_multiplier = 2.0;
                    }
                    WheelDelta::Pages(vec) => {
                        debug!("Scrolling by pages");
                        vec_y = vec.y;
                        scroll_multiplier = 1.0;
                    }
                }

                camera_distance += vec_y * scroll_multiplier;
                if camera_distance() <= MIN_CAM_DIST {
                    camera_distance.set(MIN_CAM_DIST);
                }
            },
            canvas {
                id: CANVAS_ID,
                style: format!(
                    "user-select: none; width: {}px; height: {}px; display: block; background: var(--bg2); cursor: {}; user-select: none; touch-action: none;",
                    size().0,
                    size().1,
                    if dragging() { "grab" } else { "default" },
                ),

                onmousedown: move |evt| {
                    dragging.set(true);
                    let c = evt.client_coordinates();
                    last_pos.set((c.x, c.y));
                },
                onmouseup: move |_| dragging.set(false),
                // onmouseleave: move |_| dragging.set(false),
            }
            // {children}
            // if let Some(mapped_scene) = mapped_scene.read().as_ref() {
            //     svg {
            //         width: "100%",
            //         height: "100%",
            //         view_box: "0 0 {size().0} {size().1}",

            //         onmousedown: move |evt| {
            //             dragging.set(true);
            //             let c = evt.client_coordinates();
            //             last_pos.set((c.x, c.y));
            //         },
            //         onmouseup: move |_| dragging.set(false),

            //         for l in &mapped_scene.lines {
            //             line {
            //                 x1: l.x1,
            //                 y1: l.y1,
            //                 x2: l.x2,
            //                 y2: l.y2,
            //                 stroke: l.color.clone(),
            //                 stroke_width: 1.0,
            //             }
            //         }

            //         for p in &mapped_scene.points {
            //             circle {
            //                 cx: p.0,
            //                 cy: p.1,
            //                 r: 4.0,
            //                 fill: "green",
            //             }
            //         }
            //     }
            // }
        }
    }
}
