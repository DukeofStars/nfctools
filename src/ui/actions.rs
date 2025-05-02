use color_eyre::Result;
use floem::{
    event::EventPropagation,
    prelude::*,
    reactive::{create_effect, SignalRead},
    style::TextOverflow,
    taffy::{AlignContent, AlignItems},
};
use schemas::{Fleet, Ship};
use tracing::{error, trace, warn};

use crate::{
    tags::{self, get_tags_from_description, Tag},
    themes::*,
    AppConfig,
};

pub fn actions_pane(
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
        .style(|s| s.width_full()),
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
        h_stack((text(&ship.hull_type), text(&ship.cost))).style(|s| {
            s.width_full().justify_content(AlignContent::SpaceBetween)
        }),
    ))
    .style(|s| {
        s.width_full()
            .border_color(TEXT)
            .border(1.0)
            .border_bottom(0.0)
    })
}
