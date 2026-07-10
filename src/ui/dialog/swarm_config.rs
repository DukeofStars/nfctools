use std::str::FromStr;

use dioxus::prelude::*;

use crate::ui::formations::swarm::SwarmCompressionConfig;

#[component]
pub fn SwarmConfigDialog(
    open: Signal<bool>,
    config: Signal<Option<SwarmCompressionConfig>>,
    running: Signal<bool>,
) -> Element {
    let mut min_dist = use_signal(|| 300f64);
    let mut num_iter = use_signal(|| 500usize);

    let mut repulsion = use_signal(|| 1f64);
    let mut attraction = use_signal(|| 0.01f64);
    let mut damping = use_signal(|| 0.9f64);

    rsx! {
        div { style: "display: flex; flex-direction: column;",
            h2 { "Compress Swarm Formation" }
            div { style: "display: grid; grid-template-columns: 70% 30%;",
                "Minimum distance between ships:"
                input {
                    value: "{min_dist}",
                    oninput: move |evt| {
                        if let Ok(parsed) = f64::from_str(&evt.value()) {
                            min_dist.set(parsed);
                        }
                    },
                }
                "Number of iterations:"
                input {
                    value: "{num_iter}",
                    oninput: move |evt| {
                        if let Ok(parsed) = usize::from_str(&evt.value()) {
                            num_iter.set(parsed);
                        }
                    },
                }
                "Attraction strength:"
                input {
                    value: "{attraction}",
                    oninput: move |evt| {
                        if let Ok(parsed) = f64::from_str(&evt.value()) {
                            attraction.set(parsed);
                        }
                    },
                }
                "Repulsion strength:"
                input {
                    value: "{repulsion}",
                    oninput: move |evt| {
                        if let Ok(parsed) = f64::from_str(&evt.value()) {
                            repulsion.set(parsed);
                        }
                    },
                }
                "Damping factor:"
                input {
                    value: "{damping}",
                    oninput: move |evt| {
                        if let Ok(parsed) = f64::from_str(&evt.value()) {
                            damping.set(parsed);
                        }
                    },
                }
            }
            button {
                disabled: running(),
                class: "button",
                style: "margin: 10px auto 0; width: 80%; height: 30px;",
                onclick: move |_| {
                    running.set(true);
                    config
                        .set(
                            Some(SwarmCompressionConfig {
                                min_dist: min_dist(),
                                iterations: num_iter(),
                                repulsion_strength: repulsion(),
                                attraction_strength: attraction(),
                                damping: damping(),
                            }),
                        );
                },
                if running() {
                    span { class: "spinner" }
                } else {
                    "Compress Swarm"
                }
            }
        }
    }
}
