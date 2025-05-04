use color_eyre::Result;
use floem::{
    self, kurbo::Size, prelude::*, reactive::SignalRead, window::WindowConfig,
    AppEvent, Application,
};
use schemas::Fleet;
use tracing::{error, info};

use crate::{
    fleet_data::FleetData,
    fleet_io::{self},
    themes::*,
    AppConfig,
};

mod actions;
mod fleet_editor;
mod fleet_list;

const FLEET_LIST_PCT: f64 = 30.0;
const ACTIONS_PANE_WIDTH: f64 = 240.0;
// Manually calculated
const FLEET_EDITOR_WIDTH: f64 = 550.0;

pub fn launch(cfg: &AppConfig) -> Result<()> {
    let window_config = WindowConfig::default().size(Size {
        // Minimum window width for proper layouting
        width: (FLEET_EDITOR_WIDTH + ACTIONS_PANE_WIDTH)
            / (100.0 - FLEET_LIST_PCT)
            * 100.0
            // Some buffer
            + 5.0,
        height: 600.0,
    });

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

    let selected_ship = create_rw_signal(None);

    Ok(h_stack((
        fleet_list::fleets_list(
            cfg,
            selected_fleet,
            selected_fleet_data,
            selected_fleet_idx,
        )?
        .style(|s| s.width_pct(FLEET_LIST_PCT).max_width_pct(FLEET_LIST_PCT)),
        fleet_editor::fleet_editor(selected_ship).style(|s| s.flex_grow(1.0)),
        actions::actions_pane(
            cfg,
            selected_fleet,
            selected_fleet_idx,
            selected_ship,
        )?
        .style(|s| {
            s.width_pct(30.0)
                .max_width(ACTIONS_PANE_WIDTH)
                .flex_grow(1.0)
        }),
    ))
    .style(body)
    .style(|s| s.width_full().height_full().margin(2).padding(2)))
}
