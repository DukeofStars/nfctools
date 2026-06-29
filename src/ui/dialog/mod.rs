use dioxus::{prelude::*};

pub mod merge_fleets;
pub mod error;

#[component]
pub fn DialogWrapper(signal: Signal<bool>, children: Element) -> Element {
    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 100;",
            hidden: !signal(),

            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    signal.set(false);
                }
            },
            onclick: move |_| {
                signal.set(false);
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
