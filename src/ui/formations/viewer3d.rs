//! 3D point/line/shape renderer for Dioxus Desktop.
//!
//! Desktop apps are native binaries (not wasm), so web-sys/wasm-bindgen DOM
//! bindings don't apply here. Instead we drive the embedded webview's canvas
//! via `document::eval`, sending it JS to run. This same approach also works
//! unmodified on the `web` target.
//!
//! Cargo.toml:
//!
//! [dependencies]
//! dioxus = { version = "0.6", features = ["desktop"] }

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn rotate_y(self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Point3 {
            x: self.x * cos + self.z * sin,
            y: self.y,
            z: -self.x * sin + self.z * cos,
        }
    }

    fn rotate_x(self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Point3 {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.y * sin + self.z * cos,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub points: Vec<Point3>,
    pub lines: Vec<(usize, usize)>,
}

impl Scene {
    pub fn cube() -> Self {
        Scene {
            points: vec![
                Point3::new(-1.0, -1.0, -1.0),
                Point3::new(1.0, -1.0, -1.0),
                Point3::new(1.0, 1.0, -1.0),
                Point3::new(-1.0, 1.0, -1.0),
                Point3::new(-1.0, -1.0, 1.0),
                Point3::new(1.0, -1.0, 1.0),
                Point3::new(1.0, 1.0, 1.0),
                Point3::new(-1.0, 1.0, 1.0),
            ],
            lines: vec![
                (0, 1), (1, 2), (2, 3), (3, 0), // back face
                (4, 5), (5, 6), (6, 7), (7, 4), // front face
                (0, 4), (1, 5), (2, 6), (3, 7), // connecting edges
            ],
        }
    }
}

/// Simple perspective projection: closer points (smaller z after camera offset)
/// get scaled up; farther points get scaled down.
fn project(p: Point3, width: f64, height: f64, fov: f64, camera_distance: f64) -> (f64, f64) {
    let z = (p.z + camera_distance).max(0.001);
    let scale = fov / z;
    let x = p.x * scale + width / 2.0;
    let y = -p.y * scale + height / 2.0; // flip y: canvas y grows downward
    (x, y)
}

const CONTAINER_ID: &str = "canvas-container";
const CANVAS_ID: &str = "scene-canvas";
const FOV: f64 = 300.0;
const CAMERA_DISTANCE: f64 = 5.0;

/// Projects the scene in Rust, then builds a JS snippet that draws the
/// already-projected 2D coordinates onto the canvas. Keeping the math in
/// Rust and only sending final pixel coordinates to JS keeps the eval
/// payload simple and avoids re-implementing the projection in JS.
///
/// `width`/`height` here are CSS pixels (the container's current size).
/// The JS sets the canvas's backing-store resolution to width/height
/// scaled by devicePixelRatio, then uses setTransform so all the drawing
/// calls below can stay in CSS-pixel coordinates regardless of DPI.
fn build_draw_js(scene: &Scene, width: f64, height: f64, angle_x: f64, angle_y: f64) -> String {
    let transformed: Vec<Point3> = scene
        .points
        .iter()
        .map(|p| p.rotate_y(angle_y).rotate_x(angle_x))
        .collect();

    let projected: Vec<(f64, f64)> = transformed
        .iter()
        .map(|p| project(*p, width, height, FOV, CAMERA_DISTANCE))
        .collect();

    let points_js = projected
        .iter()
        .map(|(x, y)| format!("[{x:.2},{y:.2}]"))
        .collect::<Vec<_>>()
        .join(",");

    let lines_js = scene
        .lines
        .iter()
        .map(|(a, b)| format!("[{a},{b}]"))
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
            for (const [a, b] of lines) {{
                ctx.beginPath();
                ctx.moveTo(points[a][0], points[a][1]);
                ctx.lineTo(points[b][0], points[b][1]);
                ctx.stroke();
            }}

            ctx.fillStyle = "\#f97316";
            for (const [x, y] of points) {{
                ctx.beginPath();
                ctx.arc(x, y, 4, 0, Math.PI * 2);
                ctx.fill();
            }}
        }})();
        "#
    )
}

#[component]
pub fn Canvas3D(size: Signal<(f64, f64)>, scene: Scene) -> Element {
    let mut angle_x = use_signal(|| 0.3_f64);
    let mut angle_y = use_signal(|| 0.0_f64);
    // Sensible fallback size in case the observer hasn't fired yet.

    // Drag state for mouse-driven rotation.
    let mut dragging = use_signal(|| false);
    let mut last_pos = use_signal(|| (0.0_f64, 0.0_f64));
    const DRAG_SENSITIVITY: f64 = -0.01;

    // Re-runs whenever size, angle_x, or angle_y change, since use_effect
    // auto-tracks any signals read inside its closure.
    use_effect(move || {
        let (width, height) = size();
        let js = build_draw_js(&scene, width, height, angle_x(), angle_y());
        document::eval(&js);
    });

    let (width, height) = size();

    rsx! {
        div { id: CONTAINER_ID,
            canvas {
                id: CANVAS_ID,
                style: "width: {width}px; height: {height}px; display: block; \
                        background: var(--bg2); cursor: grab; \
                        user-select: none; touch-action: none;",

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
                        angle_y += dx * DRAG_SENSITIVITY;
                        angle_x += dy * DRAG_SENSITIVITY;
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
        }
    }
}