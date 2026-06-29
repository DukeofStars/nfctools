use dioxus::prelude::*;
use dioxus_primitives::switch::{self, SwitchProps, SwitchThumbProps};

#[cfg(feature = "bundle")]
pub static STYLE: &'static str = include_str!("./style.css");

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    #[cfg(not(feature = "bundle"))]
    let style = rsx! {
        document::Stylesheet { href: asset!("./style.css") }
    };
    #[cfg(feature = "bundle")]
    let style = rsx! {};
    rsx! {
        {style}
        switch::Switch {
            class: "dx-switch",
            checked: props.checked,
            default_checked: props.default_checked,
            disabled: props.disabled,
            required: props.required,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SwitchThumb(props: SwitchThumbProps) -> Element {
    rsx! {
        switch::SwitchThumb { class: "dx-switch-thumb", attributes: props.attributes, {props.children} }
    }
}
