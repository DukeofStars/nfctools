use color_eyre::Result;
use floem::{
    self, prelude::*, reactive::SignalRead, window::WindowConfig, AppEvent,
    Application,
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
mod fleet_list;

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
        fleet_list::fleets_list(
            cfg,
            selected_fleet,
            selected_fleet_data,
            selected_fleet_idx,
        )?
        .style(|s| s.width_pct(40.0).max_width_pct(40.0)),
        text("Fleet Editor").style(|s| s.flex_grow(1.0)).style(h1),
        actions::actions_pane(cfg, selected_fleet, selected_fleet_idx)?
            .style(|s| s.width_pct(30.0).max_width(240)),
    ))
    .style(body)
    .style(|s| s.width_full().height_full().margin(2).padding(2)))
}
