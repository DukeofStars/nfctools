use std::ops::DerefMut;
use std::time::Duration;

use dioxus::{
    desktop::{use_muda_event_handler, Config, WindowBuilder},
    prelude::*,
};
use futures::StreamExt;
use palette::{
    encoding::{self, Srgb},
    rgb::Rgb,
    Hsv, IntoColor,
};
use schemas::Ship;

use crate::{
    audio::AUDIO_HANDLER,
    components::color_picker::ColorPicker,
    config::load_app_config,
    fleet_data::FleetData,
    fleet_io::read_fleet,
    load_fleets,
    spawn_async::spawn_async,
    tags::{Color, TAGS_REPO, Tag},
    ui::{dialog::{DialogWrapper, error::{ErrorDialog, ErrorType}, merge_fleets::MergeFleetsDialog, settings::SettingsDialog, spinner::SpinnerDialog}, fleet_editor::ShipEditor},
};

#[component]
pub fn FleetList() -> Element {
    let mut fleets = use_resource(async move || {
        // Load app configuration first
        spawn_async(load_app_config).await.unwrap();
        spawn_async(crate::tags::init_tags).await;
        // Then load fleets (load_fleets requires APP_CONFIG to be set)
        spawn_async(|| load_fleets::load_fleets(true)).await
    });

    let mut selected_fleet_data = use_signal(|| None::<FleetData>);
    let mut selected_fleet_idx = use_signal(|| None::<usize>);

    let mut loading_fleet = use_signal(|| false);

    let mut selected_ship = use_signal(|| None::<Ship>);
    let mut selected_ship_idx = use_signal(|| None::<usize>);

    let mut description = use_signal(String::new);
    let mut tags = use_signal(|| {
        let binding = description.read();
        let desc = binding.as_str();
        let tags = crate::tags::get_tags_from_description(desc);
        match tags {
            Ok((tags, _)) => tags,
            Err(err) => {
                warn!(?err, "Failed to retrieve tags from fleet description");
                vec![]
            }
        }
    });

    let mut tags_dirty = use_signal(|| false);

    let mut merge_fleets_dialog_open = use_signal(|| false);

    let mut show_error_dialog = use_signal(|| false);
    let mut err_title = use_signal(String::new);
    let mut err_message = use_signal(String::new);
    let mut err_type = use_signal(|| ErrorType::User);

    macro_rules! error_popup {
        ($title:expr, $msg:expr, $type:expr) => {
            {
                err_title.set(String::from($title));
                err_message.set(String::from($msg));
                err_type.set($type);
                show_error_dialog.set(true);
            }
        }
    }

    let mut spinner_title = use_signal(String::new);
    let mut show_spinner_dialog = use_signal(|| false);

    macro_rules! show_spinner {
        () => {{
            spinner_title.clear();
            show_spinner_dialog.set(true);
        }};
        ($title:expr) => {{
            spinner_title.set(String::from($title));
            show_spinner_dialog.set(true);
        }}
    }

    let mut show_settings_dialog = use_signal(|| false);

    let menu_handler =
        use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
            while let Some(action) = rx.next().await {
                match action.as_str() {
                    "fleets-reload" => {
                        info!("Reloading fleets");
                        show_spinner!("Reloading fleets");
                        fleets.set(None);
                        selected_fleet_data.set(None);
                        selected_fleet_idx.set(None);
                        selected_ship.set(None);
                        selected_ship_idx.set(None);
                        let new_fleets =
                            spawn_async(|| load_fleets::load_fleets(false))
                                .await;
                        fleets.set(Some(new_fleets));
                        show_spinner_dialog.set(false);
                    }
                    "edit-preferences" => {
                        show_settings_dialog.set(true);
                    }
                    "tools-winpred" => {
                        let dom = VirtualDom::new(
                            crate::ui::win_predictor::WinPredictor,
                        );
                        let config = Config::new().with_menu(None).with_window(
                            WindowBuilder::new().with_title(format!(
                                "NebTools v{} @dukeofstars",
                                env!("CARGO_PKG_VERSION")
                            )),
                        );

                        dioxus::desktop::window().new_window(dom, config);
                    }
                    "tools-merge" => {
                        if !selected_fleet_idx.read().is_some() {
                            error_popup!("No fleet selected", "Cannot merge 0 fleets", ErrorType::User);
                        } else {
                            merge_fleets_dialog_open.set(true);
                        }
                    }
                    "help-open-log" => {
                        if let Some(path) = crate::LOG_FILE_PATH.clone() {
                            show_spinner!("Opening log file directory");
                            let path = path.parent().unwrap();
                            let mut cmd = std::process::Command::new("cmd.exe");
                            cmd.args(["/c", "start", ""]);
                            cmd.arg(path);
                            let _ = cmd.spawn();
                            show_spinner_dialog.set(false);
                        } else {
                            error_popup!("Log file does not exist", "", ErrorType::Warn);
                            warn!("Log file path does not exist")
                        }
                    }
                    _ => {}
                }
            }
        });
    use_muda_event_handler(move |event| {
        menu_handler.send(event.id().0.clone())
    });

    // When description is updated, such as when selecting a fleet,
    // read and update the tags.
    use_effect(move || {
        let binding = description.read();
        let desc = binding.as_str();
        match crate::tags::get_tags_from_description(desc) {
            Ok((new_tags, _)) => {
                // Only update if tags actually differ, to avoid spurious writes
                if *tags.peek() != new_tags {
                    tags.set(new_tags);
                }
            }
            Err(err) => {
                warn!(?err, "Failed to retrieve tags from fleet description");
            }
        }
    });
    // When the tags are updated, insert them into the description.
    use_effect(move || {
        let tags = tags.read();

        if !tags_dirty() {
            return;
        }
        tags_dirty.set(false);

        let desc =
            crate::tags::get_tags_from_description(description.peek().as_str())
                .map(|x| x.1)
                .unwrap_or_default();

        let new_desc = if tags.is_empty() {
            desc
        } else {
            format!(
                "Tags: {}\n{}",
                tags.iter()
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

    let mut prev_path = use_signal(|| None);
    // When the selected_fleet_data is changed, asynchronously load the fleet.
    let mut selected_fleet = use_resource(move || async move {
        if let Some(fleet_data) = selected_fleet_data.as_ref() {
            loading_fleet.set(true);
            crate::menubar::MENUBARS.with_borrow(|menubars| {
                if let Some(menubars) = menubars.as_ref() {
                    menubars.tools_merge.set_enabled(true);
                }
            });
            selected_ship.set(None);
            selected_ship_idx.set(None);
            let fleet_path = fleet_data.path.clone();
            let fleet = spawn_async(|| read_fleet(fleet_path));
            let fleet = fleet.await;
            if Some(fleet_data.path.clone()) == prev_path() {
                loading_fleet.set(false);
                return fleet.ok();
            }
            if let Some(desc) =
                fleet.as_ref().ok().and_then(|f| f.description.as_ref())
            {
                *description.write() = desc.clone();
            }
            prev_path.set(Some(fleet_data.path.clone()));
            loading_fleet.set(false);
            fleet.ok()
        } else {
            crate::menubar::MENUBARS.with_borrow(|menubars| {
                if let Some(menubars) = menubars.as_ref() {
                    menubars.tools_merge.set_enabled(false);
                }
            });
            None
        }
    });

    // When the selected_ship is updated, save the fleet.
    use_effect(move || {
        let ship = selected_ship.read();
        if let Some(ship) = ship.as_ref() {
            let fleet_data_r = selected_fleet_data.read();
            let fleet_data = fleet_data_r
                .as_ref()
                .expect("Ship updated without any fleet being selected");
            let mut fleet_w = selected_fleet.write();
            let fleet = fleet_w
                .as_mut()
                .unwrap()
                .as_mut()
                .expect("Ship updated without any fleet being selected");
            let ship_idx = selected_ship_idx
                .read()
                .expect("Ship updated without any ship idx being set");
            fleet.ships.as_mut().unwrap().ship.as_mut().unwrap()[ship_idx] =
                ship.clone();
            match crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet) {
                Ok(_) => {},
                Err(err) => {
                    error_popup!("Failed to write fleet file", format!("{:?}", err), ErrorType::Warn);
                    error!("Failed to write fleet file: {:?}", err);
                }
            };
        }
    });

    // When the description is updated, save it to the fleet.
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
        fleet.description = Some(desc.clone());

        // Update fleet_data in fleets list (fleet_data_r is a clone).
        let mut fleets = fleets.as_mut();
        if let Some(fleets) = fleets.as_mut() {
            if let Ok(fleets) = fleets.deref_mut() {
                let fleet_data = fleets
                    .get_mut(
                        selected_fleet_idx()
                            .expect("Fleet updated when no fleet selected"),
                    )
                    .expect("Invalid selected_fleet_idx");
                fleet_data.description = desc;
            }
        }

        match crate::fleet_io::write_fleet(fleet_data.path.clone(), fleet) {
                Ok(_) => {},
                Err(err) => {
                    error_popup!("Failed to write fleet file", format!("{:?}", err), ErrorType::Warn);
                    error!("Failed to write fleet file: {:?}", err);
                }
            };
    });

    let mut secondary_selected_fleet_idxs = use_signal(|| Vec::<usize>::new());

    let mut search_text = use_signal(String::new);
    let search_filters = use_memo(move || {
        // Reset all selections on search
        selected_fleet.set(None);
        selected_fleet_idx.set(None);
        selected_ship.set(None);
        selected_ship_idx.set(None);
        selected_fleet_data.set(None);
        secondary_selected_fleet_idxs.clear();

        crate::search::parse_search_text(search_text())
    });

    rsx! {
        DialogWrapper { signal: show_settings_dialog,
            if show_settings_dialog() {
                SettingsDialog { signal: show_settings_dialog }
            } else {

            }
        }
        DialogWrapper { signal: show_spinner_dialog, non_exitable: true,
            if show_spinner_dialog() {
                SpinnerDialog { title: spinner_title() }
            } else {

            }
        }
        DialogWrapper { signal: merge_fleets_dialog_open,
            if merge_fleets_dialog_open() {
                {
                    let fleets = fleets.read();
                    let Some(Ok(all_fleets)) = fleets.as_ref() else {
                        warn!("Tried to open merge dialog but fleets not loaded. This is a bug");
                        return rsx! {};
                    };

                    // Indexes of fleets to be merged
                    let mut fleet_idxs = secondary_selected_fleet_idxs();
                    if let Some(idx) = selected_fleet_idx() {
                        fleet_idxs.push(idx);
                    }
                    fleet_idxs.dedup();

                    // Fleets to merge
                    let mut fleets = vec![];
                    for idx in fleet_idxs {
                        fleets.push(all_fleets[idx].clone());
                    }

                    // Sort fleets alphabetically
                    fleets.sort_by(|a, b| a.name.cmp(&b.name));

                    rsx! {
                        MergeFleetsDialog { fleets, signal: merge_fleets_dialog_open }
                    }
                }
            } else {

            }
        }
        DialogWrapper { signal: show_error_dialog,
            if show_error_dialog() {
                ErrorDialog {
                    signal: show_error_dialog,
                    title: err_title(),
                    message: err_message(),
                    error_type: err_type(),
                }
            } else {

            }
        }
        div {
            style: "
            display: grid;
            grid-template-columns: 24% 1% 50% 1% 24%;
            overflow: hidden;
            height: 100vh;",
            class: "hide-scroll",
            // Fleets List
            div { display: "flex", flex_direction: "column", min_height: 0,
                h2 { margin: 0, padding: 0, flex_shrink: 0, "Fleets" }
                input {
                    value: "{search_text}",
                    style: "margin: 0px; margin-bottom: 2px;",
                    id: "search-bar",
                    placeholder: "Search fleets",
                    oninput: move |evt| { search_text.set(evt.value()) },
                }
                div {
                    style: "
                    overflow-y: auto;
                    display: grid;
                    flex: 1;
                    min-height: 0;
                    align-content: start;
                    height: 100%;
                    ",
                    class: "hide-scroll",
                    match fleets.read().as_ref() {
                        Some(Ok(fleets)) => rsx! {
                            for (idx , fleet) in fleets.iter().enumerate() {
                                {
                                    let fleet = fleet.clone();
                                    if !search_filters.read().matches(&fleet) {
                                        return rsx! {};
                                    }

                                    let selected = use_memo(move || {
                                        selected_fleet_idx() == Some(idx)
                                            || secondary_selected_fleet_idxs.iter().any(|idx2| *idx2 == idx)
                                    });
                                    rsx! {
                                        button {
                                            onmouseenter: move |_| {
                                                if !selected() {
                                                    AUDIO_HANDLER.play_hover_sound();
                                                }
                                            },
                                            padding: 0,
                                            margin: 0,
                                            display: "flex",
                                            flex_direction: "row",
                                            justify_content: "space-between",
                                            align_items: "center",
                                            key: "{fleet.path.display()}",
                                            class: if selected() { "list-button selected" } else { "list-button" },
                                            onclick: move |evt| {
                                                let mods = evt.modifiers();
                                                if mods.ctrl() && selected_fleet_idx().is_some() {
                                                    secondary_selected_fleet_idxs.push(selected_fleet_idx().unwrap());
                                                } else if mods.shift() && selected_fleet_idx().is_some() {
                                                    let idx1 = idx;
                                                    let idx2 = selected_fleet_idx().unwrap();

                                                    let min = if idx1 > idx2 { idx2 } else { idx1 };
                                                    let max = if idx1 < idx2 { idx2 } else { idx1 };

                                                    for idx in min..=max {
                                                        secondary_selected_fleet_idxs.push(idx);
                                                    }
                                                } else {
                                                    secondary_selected_fleet_idxs.clear();
                                                }
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
            div { display: "flex", flex_direction: "column", min_height: 0,
                div {
                    {
                        match selected_fleet.read().as_ref() {
                            Some(Some(fleet)) => rsx! {
                                div {
                                    h2 { "{fleet.name}" }
                                    h4 { "Tags" }
                                    Tags { key: fleet.path, tags, tags_dirty }
                                    h4 { "Description" }
                                    textarea {
                                        height: "200px",
                                        value: {
                                            crate::tags::get_tags_from_description(description.read().as_str())
                                                .map(|x| x.1)
                                                .unwrap_or_default()
                                        },
                                        oninput: move |evt| {
                                            description
                                                .set(
                                                    format!(
                                                        "Tags: {}\n{}",
                                                        tags
                                                            .iter()
                                                            .map(|tag| {
                                                                format!(
                                                                    "<color=#{:02x}{:02x}{:02x}>{}</color>",
                                                                    tag.color.red,
                                                                    tag.color.green,
                                                                    tag.color.blue,
                                                                    tag.name,
                                                                )
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .join(" "),
                                                        evt.value(),
                                                    ),
                                                )
                                        },
                                    }
                                }
                            },
                            _ => rsx! { "no fleet selected" },
                        }
                    }
                }
                div { style: "flex: 1; min-height: 0; display: flex; flex-direction: column;",
                    h3 { "Ships" }
                    div {
                        style: "flex: 1; overflow-y: auto; min-height: 0; display: grid; align-content: start;",
                        class: "hide-scroll",
                        if loading_fleet() {
                            "Loading fleet..."
                        } else {
                            match selected_fleet.read().as_ref() {
                                Some(Some(fleet)) => rsx! {
                                    for (idx , ship) in fleet
                                        .ships
                                        .iter()
                                        .flat_map(|ships| ships.ship.iter().map(|iter| iter.iter()))
                                        .flatten()
                                        .enumerate()
                                    {
                                        {

                                            let ship = ship.clone();
                                            let selected = use_memo(move || Some(idx) == selected_ship_idx());
                                            rsx! {
                                                button {
                                                    onmouseenter: move |_| {
                                                        if !selected() {
                                                            AUDIO_HANDLER.play_hover_sound();
                                                        }
                                                    },
                                                    display: "flex",
                                                    flex_direction: "column",
                                                    justify_content: "space-between",
                                                    padding: 0,
                                                    margin: 0,
                                                    text_align: "left",
                                                    height: "40px",
                                                    class: if selected() { "list-button selected" } else { "list-button" },
                                                    onclick: move |_| {
                                                        trace!("Selecting ship {}", ship.name);
                                                        selected_ship.set(Some(ship.clone()));
                                                        selected_ship_idx.set(Some(idx));
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
    let mut new_tag_name = use_signal(String::new);
    let mut new_tag_color =
        use_signal(|| palette::Hsv::<Srgb, f64>::new(1.0, 1.0, 1.0));

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
                    let tag = Tag {
                        name: name.to_string(),
                        color: Color(color),
                    };
                    tags.write()
                        .push(tag);

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
