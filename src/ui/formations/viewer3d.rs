use dioxus::{html::geometry::WheelDelta, prelude::*};
use nalgebra::{Isometry3, Perspective3, Vector3};

pub type Point3 = nalgebra::Point3<f64>;

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub points: Vec<Point3>,
    pub lines: Vec<(usize, usize)>,
    /// Draws a circle around these points
    pub highlight_points: Vec<(usize, String)>, // Point index, and colour.
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
fn project_lines_js(
    points: &[Point3],
    edges: &[(usize, usize)],
    view: &Isometry3<f64>,
    projection: &Perspective3<f64>,
    width: f64,
    height: f64,
) -> String {
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
                format!("[{x1:.2},{y1:.2},{x2:.2},{y2:.2}]")
            })
        })
        .collect::<Vec<_>>()
        .join(",")
}

const CONTAINER_ID: &str = "canvas-container";
const CANVAS_ID: &str = "scene-canvas";
const CAMERA_DISTANCE: f64 = 1000.0;
const HIGHLIGHT_RADIUS: f64 = 7.0;

fn build_draw_js(
    scene: &Scene,
    width: f64,
    height: f64,
    pitch: f64,
    yaw: f64,
    camera_distance: f64,
    mut mapped_points: Signal<Vec<(f64, f64)>>,
) -> String {
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
    let points_js = points
        .iter()
        .map(|(x, y)| format!("[{x:.2},{y:.2}]"))
        .collect::<Vec<_>>()
        .join(",");
    mapped_points.set(points);
    let lines_js = project_lines_js(
        &scene.points,
        &scene.lines,
        &view,
        &projection,
        width,
        height,
    );

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

    let grid_lines_js = project_lines_js(
        &grid_points,
        &grid_lines,
        &view,
        &projection,
        width,
        height,
    );

    let highlight_points_js = scene
        .highlight_points
        .iter()
        .map(|(idx, col)| format!("[{idx},\"{col}\"]"))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"
        (function() {{
            const canvas = document.getElementById("{CANVAS_ID}");
            if (!canvas) return;
            const ctx = canvas.getContext("2d");
            const points = [{points_js}];
            const lines = [{lines_js}];
            const grid_lines = [{grid_lines_js}];
            const highlight_points = [{highlight_points_js}];

            // Resize the backing store to match CSS size * DPI, only when it
            // actually changed (resizing the buffer clears it as a side effect).
            const dpr = window.devicePixelRatio || 1;
            const targetW = Math.round({width} * dpr);
            const targetH = Math.round({height} * dpr);
            if (canvas.width !== targetW || canvas.height !== targetH) {{
                canvas.width = targetW;
                canvas.height = targetH;
            }}
            // All drawing below stays in CSS-pixel coordinates; this transform
            // maps them onto the (possibly higher-res) backing store.
            ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

            ctx.clearRect(0, 0, {width}, {height});
            ctx.fillStyle = "\#0a0a0a";
            ctx.fillRect(0, 0, {width}, {height});

            ctx.strokeStyle = "\#4ade80";
            ctx.lineWidth = 1.5;
            for (const [x1, y1, x2, y2] of lines) {{
                ctx.beginPath();
                ctx.moveTo(x1, y1);
                ctx.lineTo(x2, y2);
                ctx.stroke();
            }}

            ctx.fillStyle = "\#101010";
            ctx.lineWidth = 1.0;
            for (const [x1, y1, x2, y2] of grid_lines) {{
                ctx.beginPath();
                ctx.moveTo(x1, y1);
                ctx.lineTo(x2, y2);
                ctx.stroke();
            }}

            ctx.fillStyle = "\#f97316";
            for (const [x, y] of points) {{
                ctx.beginPath();
                ctx.arc(x, y, 4, 0, Math.PI * 2);
                ctx.fill();
            }}

            for (const [idx, col] of highlight_points) {{
                ctx.strokeStyle = col;
                ctx.beginPath();
                ctx.arc(points[idx][0], points[idx][1], {HIGHLIGHT_RADIUS}, 0, Math.PI * 2);
                ctx.stroke();
            }}
        }})();
        "#
    )
}

const MAX_PITCH: f64 = std::f64::consts::FRAC_PI_2 - 0.001;
const MIN_CAM_DIST: f64 = 100.0;

#[component]
pub fn Canvas3D(
    size: Signal<(f64, f64)>,
    scene: Signal<Option<Scene>>,
    mapped_points: Signal<Vec<(f64, f64)>>,
    children: Element,
) -> Element {
    let mut pitch = use_signal(|| 0.3_f64);
    let mut yaw = use_signal(|| 0.0_f64);
    let mut camera_distance = use_signal(|| CAMERA_DISTANCE);

    let mut dragging = use_signal(|| false);
    let mut last_pos = use_signal(|| (0.0_f64, 0.0_f64));
    const DRAG_SENSITIVITY: f64 = 0.01;

    use_effect(move || {
        let (width, height) = size();
        let js = build_draw_js(
            scene.read().as_ref().unwrap(),
            width,
            height,
            pitch(),
            yaw(),
            camera_distance(),
            mapped_points,
        );
        document::eval(&js);
    });

    let (width, height) = size();

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
                    "width: {width}px; height: {height}px; display: block; background: var(--bg2); cursor: {}; user-select: none; touch-action: none;",
                    if dragging() { "grab" } else { "default" },
                ),

                onmousedown: move |evt| {
                    dragging.set(true);
                    let c = evt.client_coordinates();
                    last_pos.set((c.x, c.y));
                },
                onmousemove: move |evt| {
                    if dragging() {
                        let c = evt.client_coordinates();
                        let (last_x, last_y) = last_pos();
                        let dx = c.x - last_x;
                        let dy = c.y - last_y;
                        yaw -= dx * DRAG_SENSITIVITY;
                        pitch.set((pitch() + dy * DRAG_SENSITIVITY).clamp(-MAX_PITCH, MAX_PITCH));
                        last_pos.set((c.x, c.y));
                    }
                },
                onmouseup: move |_| dragging.set(false),
                // Releasing outside the canvas only stops the drag once the
                // cursor re-enters and leaves again, since mouseup only fires
                // over the element itself -- this is the simple tradeoff for
                // not wiring up a window-level listener via eval.
                onmouseleave: move |_| dragging.set(false),
            }
            {children}
        }
    }
}
