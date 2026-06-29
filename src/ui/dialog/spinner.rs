use dioxus::prelude::*;

#[component]
pub fn SpinnerDialog(title: Option<String>) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column",
            if let Some(title) = title {
                h2 { margin: "0 auto 0", "{title}" }
            }
            span { style: "margin: 10px auto 0;", class: "spinner" }
        }
    }
}