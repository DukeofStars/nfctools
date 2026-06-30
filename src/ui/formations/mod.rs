use dioxus::prelude::*;
use schemas::Fleet;

mod viewer3d;

use viewer3d::Canvas3D;

use crate::ui::formations::viewer3d::Scene;

#[component]
pub fn FleetFormationViewer(fleet: Resource<Option<Fleet>>) -> Element {
    let mut canvas_size = use_signal(|| (0f64, 0f64));
    
    rsx! {
        h3 { "Formations" }
        div {
            style: "width: 50vw; height: 50vh;",
            onresize: move |evt: ResizeEvent| {
                let Ok(size) = evt.get_border_box_size() else { return };
                canvas_size.set((size.width, size.height));
            },
            Canvas3D { size: canvas_size, scene: Scene::cube() }
        }
    }
}