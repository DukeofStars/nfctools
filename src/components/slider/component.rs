use dioxus::prelude::*;
use dioxus_primitives::slider::{
    self, SliderProps, SliderRangeProps, SliderThumbProps, SliderTrackProps,
};

#[cfg(feature = "bundle")]
pub const STYLE: &str = include_str!("./style.css");

#[component]
pub fn Slider(props: SliderProps) -> Element {
    #[cfg(feature = "bundle")]
    let stylesheet = rsx! {};
    #[cfg(not(feature = "bundle"))]
    let stylesheet = rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
    };

    rsx! {
        {stylesheet}
        slider::Slider {
            class: "dx-slider",
            value: props.value,
            default_value: props.default_value,
            min: props.min,
            max: props.max,
            step: props.step,
            disabled: props.disabled,
            horizontal: props.horizontal,
            inverted: props.inverted,
            on_value_change: props.on_value_change,
            label: props.label,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SliderTrack(props: SliderTrackProps) -> Element {
    rsx! {
        slider::SliderTrack { class: "dx-slider-track", attributes: props.attributes, {props.children} }
    }
}

#[component]
pub fn SliderRange(props: SliderRangeProps) -> Element {
    rsx! {
        slider::SliderRange { class: "dx-slider-range", attributes: props.attributes, {props.children} }
    }
}

#[component]
pub fn SliderThumb(props: SliderThumbProps) -> Element {
    rsx! {
        slider::SliderThumb {
            class: "dx-slider-thumb",
            index: props.index,
            attributes: props.attributes,
            {props.children}
        }
    }
}
