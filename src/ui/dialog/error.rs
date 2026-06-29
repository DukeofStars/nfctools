use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[allow(unused)]
pub enum ErrorType {
    Fatal,
    User,
    Warn,
}

#[component]
pub fn ErrorDialog(
    signal: Signal<bool>,
    title: String,
    message: String,
    error_type: ErrorType,
) -> Element {
    rsx! {
        match error_type {
            ErrorType::Fatal | ErrorType::Warn => rsx! {
                h2 { "Error: {title}" }
            },
            ErrorType::User => rsx! {
                h2 { "{title}" }
            },
        }
        "{message}"
        div { display: "flex",
            button {
                class: "button",
                style: "margin: 10px auto 0; width: 80%; height: 30px;",
                onclick: move |_| { signal.set(false) },
                "Ok"
            }
        }
    }
}
