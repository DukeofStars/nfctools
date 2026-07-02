use dioxus::prelude::*;

pub mod backup;
pub mod error;
pub mod merge_fleets;
pub mod settings;
pub mod spinner;
pub mod swarm_config;

#[component]
pub fn DialogWrapper(
    signal: Signal<bool>,
    children: Element,
    non_exitable: Option<bool>,
) -> Element {
    let non_exitable = non_exitable.is_some_and(|x| x);
    rsx! {
        div {
            oncontextmenu: move |evt| evt.prevent_default(),
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 100;",
            hidden: !signal(),

            onkeydown: move |e| {
                if e.key() == Key::Escape && !non_exitable {
                    signal.set(false);
                }
            },
            tabindex: "0",

            div {
                style: "
                position: absolute;
                top: 50%; left: 50%;
                transform: translate(-50%, -50%);
                background: var(--bg2);
                padding: 2rem;
                border-radius: 8px;
                min-width: 300px;
                z-index: 101;",
                onclick: move |e| e.stop_propagation(),
                {children}
            }
        }
    }
}
