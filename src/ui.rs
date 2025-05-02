use color_eyre::{eyre::Context, Result};
use floem::{
    event::EventPropagation,
    prelude::*,
    reactive::{create_effect, SignalRead},
    style::TextOverflow,
    taffy::{AlignContent, AlignItems},
    window::WindowConfig,
    AppEvent, Application,
};
use glob::Pattern;
use schemas::{Fleet, Ship};
use styles::*;
use tracing::{error, info, trace, warn};

use crate::{
    fleet_data::FleetData,
    fleet_io::{self, read_fleet},
    load_fleets,
    tags::{self, get_tags_from_description, Tag},
    AppConfig,
};

pub fn launch(cfg: &AppConfig) -> Result<()> {
    let window_config = WindowConfig::default();

    let selected_fleet = create_rw_signal(None);
    let selected_fleet_data = create_rw_signal(None);

    let root_view = main_window(&cfg, selected_fleet, selected_fleet_data)?;

    let app = Application::new()
        // Save current fleet on exit
        .on_event(move |event| match event {
            AppEvent::WillTerminate => {
                info!("Saving current fleet");

                let binding = selected_fleet_data.read();
                let binding = binding.borrow();
                let Some(fleet_data) = binding.as_ref() else {
                    return;
                };

                let binding = selected_fleet.read();
                let binding = binding.borrow();
                let Some(fleet) = binding.as_ref() else {
                    return;
                };

                let res = fleet_io::save_fleet_data(fleet_data, fleet);
                if let Err(err) = res {
                    error!("Failed to save fleet data: {}", err);
                }
            }
            _ => {}
        })
        .window(move |_| root_view, Some(window_config));

    app.run();

    Ok(())
}

fn main_window(
    cfg: &AppConfig,
    selected_fleet: RwSignal<Option<Fleet>>,
    selected_fleet_data: RwSignal<Option<FleetData>>,
) -> Result<impl IntoView> {
    let selected_fleet_idx = create_rw_signal(0_usize);

    Ok(h_stack((
        fleets_list(
            cfg,
            selected_fleet,
            selected_fleet_data,
            selected_fleet_idx,
        )?
        .style(|s| s.width_pct(40.0).max_width_pct(40.0)),
        text("Fleet Editor").style(|s| s.flex_grow(1.0)).style(h1),
        actions_pane(cfg, selected_fleet, selected_fleet_idx)?
            .style(|s| s.width_pct(30.0).max_width(240)),
    ))
    .style(body)
    .style(|s| s.width_full().height_full().margin(2).padding(2)))
}

fn fleets_list(
    cfg: &AppConfig,
    selected_fleet: RwSignal<Option<Fleet>>,
    selected_fleet_data: RwSignal<Option<FleetData>>,
    selected_fleet_idx: RwSignal<usize>,
) -> Result<impl IntoView> {
    let excluded_patterns = cfg
        .excluded_dirs
        .iter()
        .map(|s| Pattern::new(s.as_str()).wrap_err("Failed to parse glob"))
        .collect::<Result<Vec<Pattern>>>()?;

    let fleets_dir = cfg.saves_dir.join("Fleets");
    let fleets = load_fleets::load_fleets(&fleets_dir, &excluded_patterns)?;
    let fleets_list = im::Vector::from_iter(fleets.into_iter());
    let fleets_list = create_rw_signal(fleets_list);

    let fleets_list_view = scroll(
        list(fleets_list.get().iter().map(|fleet| fleet_list_item(fleet)))
            .style(width_full)
            .on_select(move |idx| {
                if let Some(idx) = idx {
                    trace!("Selecting fleet {idx}");

                    // Save current fleet
                    'save: {
                        let binding = selected_fleet_data.read_untracked();
                        let binding = binding.borrow();
                        let Some(fleet_data) = binding.as_ref() else {
                            break 'save;
                        };

                        let binding = selected_fleet.read_untracked();
                        let binding = binding.borrow();
                        let Some(fleet) = binding.as_ref() else {
                            break 'save;
                        };

                        info!("Saving current fleet");

                        let res = fleet_io::save_fleet_data(fleet_data, fleet);
                        if let Err(err) = res {
                            error!("Failed to save fleet data: {}", err);
                        }
                    }

                    // Should never fail, but no harm in not panicking.
                    if let Some(fleet_data) =
                        fleets_list.get_untracked().get(idx)
                    {
                        selected_fleet_data.set(Some(fleet_data.clone()));
                        let fleet = match read_fleet(&fleet_data.path) {
                            Ok(fleet) => fleet,
                            Err(e) => {
                                // TODO: implement error handling.
                                warn!(
                                    "Failed to read fleet '{}': {}",
                                    fleet_data.path.display(),
                                    e
                                );

                                return;
                            }
                        };
                        selected_fleet.set(Some(fleet));
                    }

                    selected_fleet_idx.set(idx);
                }
            }),
    )
    .style(|s| s.flex_basis(0.0).min_height(0.0).flex_grow(1.0));

    Ok(v_stack((
        h_stack((
            text("Fleets").style(h1),
            button("Refresh").on_click(move |_event| -> EventPropagation {
                let fleets = match load_fleets::load_fleets(
                    &fleets_dir,
                    &excluded_patterns,
                ) {
                    Ok(fleets) => fleets,
                    Err(err) => {
                        error!("Failed to load fleets: {}", err);
                        return EventPropagation::Stop;
                    }
                };

                fleets_list.set(im::Vector::from_iter(fleets.into_iter()));

                EventPropagation::Stop
            }),
        ))
        .style(|s| s.justify_content(AlignContent::SpaceBetween)),
        fleets_list_view,
    )))
}

fn fleet_list_item(fleet_data: &FleetData) -> impl IntoView {
    h_stack((
        text(&fleet_data.name).style(|s| {
            s.max_width_pct(70.0).text_overflow(TextOverflow::Ellipsis)
        }),
        text(fleet_data.short_path.display()),
    ))
    .style(|s| {
        s.justify_content(AlignContent::SpaceBetween)
            .width_full()
            .border_color(Color::BLACK)
            .border(1.0)
            .border_bottom(0.0)
    })
}

fn actions_pane(
    _cfg: &AppConfig,
    selected_fleet: RwSignal<Option<Fleet>>,
    selected_fleet_idx: RwSignal<usize>,
) -> Result<impl IntoView> {
    let tags_repo = create_rw_signal(tags::load_tags()?);

    // === Editable parameters ===

    let tag_name = create_rw_signal(String::new());

    let color_r = create_rw_signal(String::new());
    let color_g = create_rw_signal(String::new());
    let color_b = create_rw_signal(String::new());

    let description = create_rw_signal(String::new());
    create_effect(move |_| {
        selected_fleet_idx.track();

        trace!("Updating description box");
        let new_desc = if let Some(selected_fleet) =
            selected_fleet.read_untracked().borrow().as_ref()
        {
            if let Some(Ok((_, desc))) = &selected_fleet
                .description
                .as_ref()
                .map(|d| get_tags_from_description(&d))
            {
                desc.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        description.set(new_desc);
    });
    create_effect(move |_| {
        let description = description.get();
        selected_fleet.update(|fleet| {
            let Some(fleet) = fleet else { return };
            if let Some(desc_raw) = &mut fleet.description {
                trace!("Updating description");
                let Ok((tags, _desc_old)) =
                    get_tags_from_description(&desc_raw)
                else {
                    warn!("Failed to parse fleet description");
                    return;
                };
                *desc_raw = format!(
                    "Tags: {}\n{}",
                    tags.iter()
                        .map(|tag| format!(
                            "<color=#{:02x}{:02x}{:02x}>{}</color>",
                            tag.color.r, tag.color.g, tag.color.b, tag.name
                        ))
                        .collect::<Vec<_>>()
                        .join(" "),
                    description,
                );
            } else {
                fleet.description = Some(description);
            };
        });
    });

    // ===========================

    create_effect(move |_| {
        tag_name.track();
        let tags_repo = tags_repo.read();
        let tags_repo = tags_repo.borrow();
        let color = tags_repo.get_tag(&tag_name.read().borrow());

        if let Some(color) = color {
            color_r.set(color.r.to_string());
            color_g.set(color.g.to_string());
            color_b.set(color.b.to_string());
        }

        ()
    });

    let tag_editor = h_stack((
        text_input(color_r).style(|s| s.width_pct(25.0)),
        text_input(color_g).style(|s| s.width_pct(25.0)),
        text_input(color_b).style(|s| s.width_pct(25.0)),
        text("PREVIEW").style(move |s| {
            s.color(Color::rgb8(
                color_r.get().parse().unwrap_or_default(),
                color_g.get().parse().unwrap_or_default(),
                color_b.get().parse().unwrap_or_default(),
            ))
            .width_pct(25.0)
            .justify_self(AlignItems::Center)
            .align_self(AlignItems::Center)
        }),
    ));
    let tag_section = v_stack((
        text("Tags").style(h2),
        h_stack((
            text_input(tag_name)
                .placeholder("Tag Name")
                .style(|s| s.width_full()),
            button("Add")
                .style(|s| s.flex_basis(0.0).flex_grow(0.0))
                .on_click(move |_| {
                    tags_repo.update(|tags_repo| {
                        tags_repo.add_tag(
                            tag_name.get(),
                            Color::rgb8(
                                color_r.get().parse().unwrap_or_default(),
                                color_g.get().parse().unwrap_or_default(),
                                color_b.get().parse().unwrap_or_default(),
                            ),
                        )
                    });

                    selected_fleet.update(|fleet| {
                        if let Some(fleet) = fleet {
                            let Ok((mut tags, desc)) =
                                get_tags_from_description(
                                    fleet
                                        .description
                                        .clone()
                                        .unwrap_or_default()
                                        .as_str(),
                                )
                            else {
                                error!("Unable to load tags from fleet");
                                return;
                            };

                            let tag = Tag {
                                name: tag_name.get(),
                                color: Color::rgb8(
                                    color_r.get().parse().unwrap_or_default(),
                                    color_g.get().parse().unwrap_or_default(),
                                    color_b.get().parse().unwrap_or_default(),
                                ),
                            };
                            tags.push(tag);

                            trace!("Inserting tags into description");
                            fleet.description = Some(format!(
                                "Tags: {}\n{}",
                                tags.iter()
                                    .map(|tag| format!(
                                        "<color=#{:02x}{:02x}{:02x}>{}</color>",
                                        tag.color.r,
                                        tag.color.g,
                                        tag.color.b,
                                        tag.name
                                    ))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                                desc,
                            ));
                        }
                    });

                    EventPropagation::Stop
                }),
        )),
        tag_editor,
    ));

    // Holy god of rust please forgive me
    let ship_list_view = scroll(
        dyn_view(move || {
            list(
                selected_fleet
                    .get()
                    .iter()
                    .filter_map(|fleet| {
                        fleet
                            .ships
                            .iter()
                            .filter_map(|ships| {
                                ships
                                    .ship
                                    .as_ref()
                                    .map(|ship| ship.iter().map(ship_list_item))
                            })
                            .next()
                    })
                    .next()
                    .unwrap_or(vec![].iter().map(ship_list_item)),
            )
            .style(|s| s.width_full().border_bottom(1.0))
        })
        .style(width_full),
    )
    .style(|s| {
        s.flex_basis(0.0)
            .min_height(0.0)
            .flex_grow(1.0)
            .width_full()
    });

    Ok(v_stack((
        text("Actions").style(h1),
        // Tags
        tag_section,
        // Description
        text("Edit Description").style(h2),
        text_input(description).style(|s| s.flex_grow(0.5)),
        text("Ships").style(h1),
        ship_list_view.style(|s| s.flex_grow(1.0)),
    ))
    .style(|s| s.width_full()))
}

fn ship_list_item(ship: &Ship) -> impl IntoView {
    v_stack((
        text(&ship.name)
            .style(h3)
            .style(|s| s.text_overflow(TextOverflow::Ellipsis).width_full()),
        h_stack((text(&ship.hull_type), text(&ship.cost)))
            .style(width_full)
            .style(|s| s.justify_content(AlignContent::SpaceBetween)),
    ))
    .style(|s| {
        s.width_full()
            .border_color(Color::BLACK)
            .border(1.0)
            .border_bottom(0.0)
    })
}

mod styles {
    use floem::style::Style;

    pub fn h1(style: Style) -> Style {
        style.font_size(24)
    }
    pub fn h2(style: Style) -> Style {
        style.font_size(18)
    }
    pub fn h3(style: Style) -> Style {
        style.font_size(16)
    }
    pub fn body(style: Style) -> Style {
        style.font_size(14)
    }

    pub fn width_full(style: Style) -> Style {
        style.width_full()
    }
}
