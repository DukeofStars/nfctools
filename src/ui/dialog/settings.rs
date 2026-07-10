use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use rfd::AsyncFileDialog;

use crate::{components::checkbox::Checkbox, system::config::APP_CONFIG};

#[component]
pub fn SettingsDialog(signal: Signal<bool>) -> Element {
    let mut config =
        use_signal(|| APP_CONFIG.get().unwrap().lock().unwrap().clone());

    let mut saving = use_signal(|| false);

    rsx! {
        div { style: "width: 70vw; height: 80vh; display: flex; flex-direction: column; justify-content: space-between",
            div {
                h2 { style: "margin: 0 auto 0; border-bottom: 3px solid var(--highlight);",
                    "Settings"
                }
                div { style: "
                margin-top: 10px;
                margin-bottom: 0px;
                display: grid;
                grid-template-columns: 30% 70%",
                    p { "Saves Directory" }
                    div { style: "display: flex; flex-direction: row",
                        input {
                            flex: "1",
                            width: "100%",
                            value: "{config.read().saves_dir.display()}",
                            oninput: move |evt| { config.write().saves_dir = PathBuf::from(evt.value()) },
                        }
                        button {
                            class: "button",
                            onclick: move |_| {
                                spawn(async move {
                                    let Some(path) = AsyncFileDialog::new().pick_folder().await else { return };
                                    config.write().saves_dir = path.path().to_path_buf();
                                });
                            },
                            "..."
                        }
                    }
                    p { "Sound Effects" }
                    div { style: "display: flex; flex-direction: row; justify-content: center;",
                        Checkbox {
                            checked: if config.read().sound_effects { CheckboxState::Checked } else { CheckboxState::Unchecked },
                            on_checked_change: move |checked| {
                                match checked {
                                    CheckboxState::Checked => config.write().sound_effects = true,
                                    CheckboxState::Indeterminate => {}
                                    CheckboxState::Unchecked => config.write().sound_effects = false,
                                }
                            },
                        }
                    }
                    p { "Use cache for fleets" }
                    div { style: "display: flex; flex-direction: row; justify-content: center;",
                        Checkbox {
                            checked: if config.read().use_fleet_cache { CheckboxState::Checked } else { CheckboxState::Unchecked },
                            on_checked_change: move |checked| {
                                match checked {
                                    CheckboxState::Checked => config.write().use_fleet_cache = true,
                                    CheckboxState::Indeterminate => {}
                                    CheckboxState::Unchecked => config.write().use_fleet_cache = false,
                                }
                            },
                        }
                    }
                }
                div { style: "margin-top: 10px; display: flex; flex-direction: column; width: 100%;",
                    div { style: "display: flex; flex-direction: row; width: 100%; justify-content: space-between;",
                        "Excluded Directories"
                        button {
                            class: "button",
                            onclick: move |_| { config.write().excluded_dirs.push(String::new()) },
                            "Add"
                        }
                    }
                    for (i , dir) in config.read().excluded_dirs.iter().enumerate() {
                        div { style: "display: flex; flex-direction: row;",
                            input {
                                value: "{dir}",
                                oninput: move |evt| {
                                    config.write().excluded_dirs[i] = evt.value();
                                },
                                onchange: move |evt| {
                                    if evt.value().is_empty() {
                                        config.write().excluded_dirs.remove(i);
                                    }
                                },
                            }
                            button {
                                class: "button",
                                onclick: move |_| {
                                    config.write().excluded_dirs.remove(i);
                                },
                                "Del"
                            }
                        }
                    }
                }
            }
            div { style: "display: flex; flex-direction: row; justify-content: center; gap: 30px;",
                button {
                    class: "button",
                    style: "width: 100px; height: 30px",
                    onclick: move |_| { signal.set(false) },
                    "Cancel"
                }
                button {
                    disabled: saving(),
                    class: "button",
                    style: "width: 100px; height: 30px",
                    onclick: move |_| {
                        spawn(async move {
                            saving.set(true);
                            *APP_CONFIG.get().unwrap().lock().unwrap() = config();
                            crate::system::config::save_app_config().unwrap();
                            saving.set(false);
                            signal.set(false);
                        });
                    },
                    if saving() {
                        span { class: "spinner" }
                    } else {
                        "Save"
                    }
                }
            }
        }
    }
}
