use slint::{ComponentHandle, Weak};
use tracing::{info, instrument, trace};

use crate::{
    error::{wrap_errorable_function, Error},
    my_error, MainWindow, WinPredictorWindow,
};

pub fn on_open_win_predictor_handler(main_window_weak: Weak<MainWindow>) -> impl Fn() {
    move || {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || open_win_predictor());
    }
}

#[instrument(skip())]
fn open_win_predictor() -> Result<(), Error> {
    info!("Initialising win predictor");
    trace!("Creating window");
    let window = WinPredictorWindow::new()
        .map_err(|err| my_error!("Failed to create win predictor window", err))?;
    {
        let window_weak = window.as_weak();
        let _ = window.on_update_prediction(move || {
            let window = window_weak.unwrap();
            let points1 = window.get_points_1();
            let points2 = window.get_points_2();
            let caps1 = window.get_caps_1();
            let caps2 = window.get_caps_2();
            let victory_points = window.get_victory_points();

            if caps1 == 0 || caps2 == 0 || victory_points == 0 || points1 == 0 || points2 == 0 {
                window.set_prediction("".into());
                return;
            }

            let ticks_until_team1_victory = (victory_points - points1) / caps1 / 2;
            let ticks_until_team2_victory = (victory_points - points2) / caps2 / 2;

            if ticks_until_team1_victory > ticks_until_team2_victory {
                window.set_prediction("Team 2 Victory".into());
            } else if ticks_until_team1_victory < ticks_until_team2_victory {
                window.set_prediction("Team 1 Victory".into());
            } else if ticks_until_team1_victory == ticks_until_team2_victory {
                window.set_prediction("Tie".into());
            }
        });
    }

    window
        .show()
        .map_err(|err| my_error!("Failed to show win predictor window", err))?;

    Ok(())
}
