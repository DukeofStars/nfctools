use std::time::Duration;
use std::sync::Arc;

use dioxus::prelude::*;
use lazy_static::lazy_static;
use palette::{Hsv, IntoColor, encoding::{self, Srgb}, rgb::Rgb};
use schemas::Ship;

use crate::{
    components::color_picker::ColorPicker, config::load_app_config, fleet_data::FleetData, fleet_io::read_fleet, load_fleets, spawn_async::spawn_async, tags::{Color, TAGS_REPO, Tag}, ui::fleet_editor::ShipEditor
};

struct AudioHandler {
    _handle: rodio::MixerDeviceSink,
    player: rodio::Player,
}
impl AudioHandler {
    const HOVER_SOUND: &[u8] = include_bytes!("../../assets/hover-click.wav");

    fn new() -> AudioHandler {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
            .expect("open default audio stream");
        let player = rodio::Player::connect_new(&handle.mixer());
        AudioHandler {
            _handle: handle,
            player,
        }
    }

    fn play_hover_sound(&self) {
        let cursor = std::io::Cursor::new(AudioHandler::HOVER_SOUND);
        let source = rodio::Decoder::try_from(cursor).unwrap();
        self.player.skip_one();
        self.player.append(source);
    }
}

lazy_static! {
    static ref AUDIO_HANDLER: Arc<AudioHandler> = Arc::new(AudioHandler::new());
}

#[component]
pub fn FleetList() -> Element {
    let fleets = use_resource(async move || {
        // Load app configuration first
        spawn_async(load_app_config).await.unwrap();
        spawn_async(crate::tags::init_tags).await;
        // Then load fleets (load_fleets requires APP_CONFIG to be set)
        spawn_async(load_fleets::load_fleets).await
    });

    let mut selected_fleet_data = use_signal(|| None::<FleetData>);

    let mut loading_fleet = use_signal(|| false);

    let mut selected_ship = use_signal(|| None::<Ship>);
    let mut selected_ship_idx = use_signal(|| None::<usize>);

    let mut description = use_signal(|| String::new());
    let mut tags = use_signal(|| {
        let binding = description.read();
        let desc = binding.as_str();
        let tags = crate::tags::get_tags_from_description(desc);
        match tags {
            Ok((tags, _)) => tags,
            Err(err) => {warn!(?err, "Failed to retrieve tags from fleet description"); vec![]},
        }
    });

    let mut tags_dirty = use_signal(|| false);
    use_effect(move || {
        let binding = description.read(); // subscribe to description
        let desc = binding.as_str();
        match crate::tags::get_tags_from_description(desc) {
            Ok((new_tags, _)) => {
                // Only update if tags actually differ, to avoid spurious writes
                if *tags.peek() != new_tags {
                    tags.set(new_tags);
                    // Don't set tags_dirty — this was an external change
                }
            },
            Err(err) => { warn!(?err, "Failed to retrieve tags from fleet description"); }
        }
    });
    use_effect(move || {
        let tags = tags.read();

        if !tags_dirty() { // only run if tags changed from UI, not from Effect 1
            return;
        }
        tags_dirty.set(false);
        
        let desc = crate::tags::get_tags_from_description(description.peek().as_str()).map(|x| x.1).unwrap_or_default();

        let new_desc = if tags.len() == 0 {
            desc
        } else {
            format!(
                "Tags: {}\n{}",
                tags
                    .iter()
                    .map(|tag| format!(
                        "<color=#{:02x}{:02x}{:02x}>{}</color>",
                        tag.color.red,
                        tag.color.green,
                        tag.color.blue,
                        tag.name
                    ))
                    .collect::<Vec<_>>()
                    .join(" "),
                desc,
            )
        };
        description.set(new_desc);
    });

    let mut selected_fleet = use_resource(move || async move {
        if let Some(fleet_data) = selected_fleet_data.as_ref() {
            loading_fleet.set(true);
            selected_ship.set(None);
            selected_ship_idx.set(None);
            let fleet_path = fleet_data.path.clone();
            let fleet = spawn_async(|| read_fleet(fleet_path));
            let fleet = fleet.await;
            if let Some(desc) = fleet.as_ref().ok().and_then(|f| f.description.as_ref()) {
                *description.write() = desc.clone();
            }
            loading_fleet.set(false);
            fleet.ok()
        } else {
            None
        }
    });

    let mut selected_fleet_idx = use_signal(|| None::<usize>);

    use_effect(move || {
        let ship = selected_ship.read();
        if let Some(ship) = ship.as_ref() {
            let fleet_data_r = selected_fleet_data.read();
            let fleet_data = fleet_data_r.as_ref().expect("Ship updated without any fleet being selected");
            let mut fleet_w = selected_fleet.write();
            let fleet = fleet_w.as_mut().unwrap().as_mut().expect("Ship updated without any fleet being selected");
            let ship_idx = selected_ship_idx.read().expect("Ship updated without any ship idx being set");
            fleet.ships.as_mut().unwrap().ship.as_mut().unwrap()[ship_idx] = ship.clone();
            crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet).expect("Failed to write fleet");
        }
    });

    use_effect(move || {
        let desc = description();
        let fleet_data_r = selected_fleet_data.read();
        let Some(fleet_data) = fleet_data_r.as_ref() else {
            return;
        };
        let mut fleet_w = selected_fleet.write();
        let Some(fleet) = fleet_w.as_mut().unwrap().as_mut() else {
            return;
        };
        fleet.description = Some(desc);
        crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet).expect("Failed to write fleet");
    });

    rsx! {
        div {
            display: "grid",
            grid_template_columns: "24% 1% 50% 1% 24%",
            overflow: "hidden",
            height: "97vh",
            // Fleets List
            div {
                display: "flex",
                flex_direction: "column",
                min_height: 0,
                flex: 1,
                h2 { margin: 0, padding: 0, flex_shrink: 0, "Fleets" }
                div {
                    style: "
                    overflow-y: auto;
                    display: grid;
                    flex: 1;
                    min_height: 0;
                    ",
                    class: "hide-scroll",
                    match fleets.read().as_ref() {
                        Some(Ok(fleets)) => rsx! {
                            for (idx , fleet) in fleets.iter().enumerate() {
                                {
                                    let fleet = fleet.clone();
                                    let selected = use_memo(move || { selected_fleet_idx() == Some(idx) });
                                    rsx! {
                                        button {
                                            onmouseenter: move |_| {
                                                AUDIO_HANDLER.play_hover_sound();
                                            },
                                            padding: 0,
                                            margin: 0,
                                            display: "flex",
                                            flex_direction: "row",
                                            justify_content: "space-between",
                                            align_items: "center",
                                            key: "{fleet.path.display()}",
                                            class: if selected() { "list-button selected" } else { "list-button" },
                                            onclick: move |_| {
                                                debug!("Selected fleet {}", fleet.name);
                                                loading_fleet.set(true);
                                                selected_fleet_data.set(Some(fleet.clone()));
                                                selected_fleet_idx.set(Some(idx));
                                            },
                                            "{fleet.name}"
                                            p { class: if selected() { "bg-text selected" } else { "bg-text" }, "{fleet.short_path.display()}" }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => {
                            warn!("Failed to load fleets: {}", err);
                            rsx! {
                                div { "Failed to load fleets" }
                            }
                        }
                        None => rsx! {
                            div { "Loading fleets…" }
                        },
                    }
                }
            }
            div {}
            // Fleet editor (middle)
            ShipEditor { ship: selected_ship }
            div {}
            div {
                display: "flex",
                flex_direction: "column",
                justify_content: "start",
                overflow: "hidden",
                div {
                    {
                        match selected_fleet.read().as_ref() {
                            Some(Some(fleet)) => rsx! {
                                div {
                                    h2 { "{fleet.name}" }
                                    h4 { "Tags" }
                                    Tags { key: fleet.name, tags, tags_dirty }
                                    h4 { "Description" }
                                    textarea {
                                        height: "200px",
                                        value: {
                                            crate::tags::get_tags_from_description(description.read().as_str())
                                                .map(|x| x.1)
                                                .unwrap_or_default()
                                        },
                                        oninput: move |evt| { description.set(evt.value()) },
                                    }
                                }
                            },
                            _ => rsx! { "no fleet selected" },
                        }
                    }
                }
                div {
                    h3 { "Ships" }
                    div {
                        overflow_y: "scroll",
                        display: "grid",
                        class: "hide-scroll",
                        if loading_fleet() {
                            "Loading fleet..."
                        } else {
                            match selected_fleet.read().as_ref() {
                                Some(Some(fleet)) => rsx! {
                                    for (idx , ship) in fleet
                                        .ships
                                        .iter()
                                        .map(|ships| ships.ship.iter().map(|iter| iter.iter()))
                                        .flatten()
                                        .flatten()
                                        .enumerate()
                                    {
                                        {

                                            let ship = ship.clone();
                                            let selected = use_memo(move || Some(idx) == selected_ship_idx());
                                            rsx! {
                                                button {
                                                    onmouseenter: move |_| {
                                                        AUDIO_HANDLER.play_hover_sound();
                                                    },
                                                    display: "flex",
                                                    flex_direction: "column",
                                                    justify_content: "space-between",
                                                    padding: 0,
                                                    margin: 0,
                                                    text_align: "left",
                                                    height: "40px",
                                                    class: format!("list-button ship-list-item {}", if selected() { "selected" } else { "" }),
                                                    onclick: move |_| {
                                                        trace!("Selecting ship {}", ship.name);
                                                        selected_ship.set(Some(ship.clone()));
                                                        selected_ship_idx.set(Some(idx.clone()));
                                                    },
                                                    "{ship.name}"
                                                    div {
                                                        display: "flex",
                                                        justify_content: "space-between",
                                                        background_color: "transparent",
                                                        p { class: "bg-text", background_color: "transparent", "{ship.hull_type}" }
                                                        p { class: "bg-text", background_color: "transparent", "{ship.cost}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                Some(None) => rsx! {
                                    div {}
                                },
                                None => rsx! {
                                    div { "Error (002)" }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Tags(tags: Signal<Vec<Tag>>, tags_dirty: Signal<bool>) -> Element {
    let mut new_tag_name = use_signal(|| String::new());
    let mut new_tag_color = use_signal(|| palette::Hsv::<Srgb, f64>::new(1.0, 1.0, 1.0));

    let mut color_picker_open = use_signal(|| false);

    let mut focus_in = use_signal(|| false);

    rsx! {
        div { display: "flex", flex_direction: "column", width: "100%",
            form {
                onsubmit: move |_| {
                    let name = new_tag_name();
                    let name = name.trim();
                    if name == "" {
                        return;
                    }

                    new_tag_name.set(String::new());

                    let color: Rgb<Srgb, f64> = new_tag_color().into_color();
                    let color: Rgb<Srgb, u8> = color.into_format();
                    tags_dirty.set(true);
                    tags.write()
                        .push(Tag {
                            name: name.to_string(),
                            color: Color(color),
                        });

                    TAGS_REPO
                        .get()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .add_tag(name.to_string(), Color(color));
                },
                div { display: "flex", flex_direction: "row", width: "100%",
                    div {
                        onfocusin: move |_| {
                            focus_in.set(true);
                        },
                        onfocusout: move |_| {
                            focus_in.set(false);

                            spawn(async move {
                                tokio::time::sleep(Duration::from_millis(50)).await;
                                if !focus_in() {
                                    color_picker_open.set(false);
                                }
                            });
                        },
                        ColorPicker {
                            open: color_picker_open(),
                            flex_grow: 0,
                            color: new_tag_color,
                            on_open_change: move |now_open| {
                                color_picker_open.set(now_open);
                            },
                            on_color_change: move |c: Hsv<encoding::Srgb, f64>| {
                                new_tag_color.set(c);
                            },
                        }
                    }
                    input {
                        r#type: "text",
                        value: "{new_tag_name}",
                        oninput: move |evt| {
                            new_tag_name.set(evt.value());
                            match TAGS_REPO.get().unwrap().lock().unwrap().get_tag(&evt.value()) {
                                Some(color) => {
                                    let color: Rgb<Srgb, f64> = color.into_format();
                                    let color: Hsv<Srgb, f64> = color.into_color();
                                    new_tag_color.set(color);
                                }
                                None => {}
                            }
                        },
                    }
                    button { class: "button", flex_grow: 0, "Add" }
                }
            }
            div { display: "grid", grid_template_columns: "25% 25% 25% 25%",
                for (idx , tag) in tags().into_iter().enumerate() {
                    {
                        let color = tag.color;
                        rsx! {
                            button {
                                class: "list-button tag-button",
                                style: format!("color: rgb({}, {}, {})", &color.red, &color.green, &color.blue),
                                onclick: move |_| {
                                    tags_dirty.set(true);
                                    tags.write().remove(idx);
                                },
                                {tag.name}
                            }
                        }
                    }
                }
            }
        }
    }
}