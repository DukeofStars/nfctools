use crate::components::slider::Slider;
use dioxus::prelude::*;

#[component]
pub fn WinPredictor() -> Element {
    #[cfg(not(feature = "bundle"))]
    return rsx! {
        // document::Stylesheet { href: crate::COMPONENT_CSS }
        // document::Stylesheet { href: crate::MAIN_CSS }

        WinPredictorInner {}
    };
    #[cfg(feature = "bundle")]
    return rsx! {
        // document::Style { {include_str!("../../../assets/main.css")} }
        // document::Style { {include_str!("../../../assets/dx-components-theme.css")} }
        // document::Style { {crate::components::dropdown_menu::STYLE} }
        // document::Style { {crate::components::color_picker::STYLE} }
        document::Style { {crate::components::slider::STYLE} }

        WinPredictorInner {}
    };
}

#[component]
fn WinPredictorInner() -> Element {
    let mut num_caps = use_signal(|| 5f64);
    rsx! {
        h1 { "Win Predictor" }
        Slider {
            value: 0.0,
        }
                // button { class: "cap-circle", "A" }
        // button { class: "cap-circle", "B" }
        // button { class: "cap-circle", "C" }
        // button { class: "cap-circle", "D" }
    }
}
