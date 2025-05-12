use color_eyre::Result;
use floem::prelude::*;
use schemas::Fleet;

use super::{
    actions, fleet_editor, fleet_list, WindowState, ACTIONS_PANE_WIDTH,
    FLEET_LIST_PCT,
};
use crate::{fleet_data::FleetData, AppConfig};

pub fn fleets_tab(
    cfg: &AppConfig,
    window_state: RwSignal<WindowState>,
    selected_fleet: RwSignal<Option<Fleet>>,
    selected_fleet_data: RwSignal<Option<FleetData>>,
) -> Result<impl IntoView> {
    let selected_fleet_idx = create_rw_signal(0_usize);
    let selected_ship_idx = create_rw_signal(0_usize);

    Ok(h_stack((
        fleet_list::fleets_list(
            cfg,
            window_state,
            selected_fleet,
            selected_fleet_data,
            selected_fleet_idx,
        )?
        .style(|s| s.width_pct(FLEET_LIST_PCT).max_width_pct(FLEET_LIST_PCT)),
        fleet_editor::fleet_editor(selected_ship_idx, selected_fleet)
            .style(|s| s.flex_grow(1.0)),
        actions::actions_pane(
            cfg,
            selected_fleet,
            selected_fleet_idx,
            selected_ship_idx,
        )?
        .style(|s| {
            s.width_pct(30.0)
                .max_width(ACTIONS_PANE_WIDTH)
                .flex_grow(1.0)
        }),
    ))
    .style(|s| s.width_full().height_full()))
}
