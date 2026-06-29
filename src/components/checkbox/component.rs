use dioxus::prelude::*;
use dioxus_primitives::checkbox::{self, CheckboxProps};
use dioxus_primitives::icon;

#[cfg(feature = "bundle")]
pub static STYLE: &'static str = include_str!("./style.css");

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    #[cfg(not(feature = "bundle"))]
    let style = rsx! {
        document::Stylesheet { href: asset!("./style.css") }
    };
    #[cfg(feature = "bundle")]
    let style = rsx! {};
    rsx! {
        {style}
        checkbox::Checkbox {
            class: "dx-checkbox",
            checked: props.checked,
            default_checked: props.default_checked,
            required: props.required,
            disabled: props.disabled,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: props.attributes,
            checkbox::CheckboxIndicator { class: "dx-checkbox-indicator",
                icon::Icon {
                    class: "dx-checkbox-check-icon",
                    width: "1rem",
                    height: "1rem",
                    path { d: "M5 13l4 4L19 7" }
                }
            }
        }
    }
}
