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
    themes::body,
    AppConfig,
};

mod actions;
mod fleet_editor;
mod fleet_list;
mod fleets_tab;
mod main_menu;
mod win_predictor;

const FLEET_LIST_PCT: f64 = 30.0;
const ACTIONS_PANE_WIDTH: f64 = 240.0;
// Manually calculated
const FLEET_EDITOR_WIDTH: f64 = 550.0;

enum WindowState {
    FleetsList {
        selected_fleet: RwSignal<Option<Fleet>>,
        selected_fleet_data: RwSignal<Option<FleetData>>,
    },
    MainMenu,
    WinPredictor,
}

pub fn launch(cfg: AppConfig) -> Result<()> {
    let window_config = WindowConfig::default()
        .title(format!(
            "NebTools v{} - By Duke of Stars (@dukeofstars)",
            env!("CARGO_PKG_VERSION")
        ))
        .size(Size {
            // Minimum window width for proper layouting
            width: (FLEET_EDITOR_WIDTH + ACTIONS_PANE_WIDTH)
            / (100.0 - FLEET_LIST_PCT)
            * 100.0
            // Some buffer
            + 5.0,
            height: 600.0,
        });

    let window_state = create_rw_signal(WindowState::FleetsList {
        selected_fleet: create_rw_signal(None),
        selected_fleet_data: create_rw_signal(None),
    });

    let root_view = root_view(cfg, window_state);

    let app = Application::new()
        // Save current fleet on exit
        .on_event(move |event| match event {
            AppEvent::WillTerminate => match *window_state.read().borrow() {
                WindowState::FleetsList {
                    selected_fleet,
                    selected_fleet_data,
                } => {
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
            },
            _ => {}
        })
        .window(move |_| root_view, Some(window_config));

    app.run();

    Ok(())
}

fn root_view(
    cfg: AppConfig,
    window_state: RwSignal<WindowState>,
) -> impl IntoView {
    dyn_view(move || match *window_state.read().borrow() {
        WindowState::FleetsList {
            selected_fleet,
            selected_fleet_data,
        } => fleets_tab::fleets_tab(
            &cfg,
            window_state,
            selected_fleet,
            selected_fleet_data,
        )
        .expect("Failed to setup window")
        .into_any(),
        WindowState::MainMenu => main_menu::main_menu(window_state).into_any(),
        WindowState::WinPredictor => {
            win_predictor::win_predictor_page().into_any()
        }
    })
    .style(body)
    .style(|s| s.width_full().height_full().padding(2))
}
