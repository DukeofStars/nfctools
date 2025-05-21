use color_eyre::{eyre::Context, Result};
use floem::{
    event::EventPropagation, prelude::*, reactive::SignalRead,
    style::TextOverflow, taffy::AlignContent,
};
use glob::Pattern;
use schemas::Fleet;
use tracing::{error, info, trace, warn};

use super::WindowState;
use crate::{
    fleet_data::FleetData,
    fleet_io::{self, read_fleet},
    load_fleets,
    themes::*,
    AppConfig,
};

pub fn fleets_list(
    cfg: &AppConfig,
    window_state: RwSignal<WindowState>,
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
            .style(|s| s.width_full())
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
            button("<-")
                .style(secondary_button)
                .on_click(move |_event| {
                    window_state.set(WindowState::MainMenu);
                    EventPropagation::Stop
                }),
            text("Fleets").style(h1),
            button("Refresh").style(primary_button).on_click(
                move |_event| -> EventPropagation {
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
                },
            ),
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
    .style(|s| s.justify_content(AlignContent::SpaceBetween).width_full())
    .style(list_item)
}
