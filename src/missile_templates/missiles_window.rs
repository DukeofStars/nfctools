use std::{path::PathBuf, rc::Rc};

use slint::{ComponentHandle, Model, VecModel, Weak};
use tracing::{debug, info, instrument, trace};

use crate::{
    error::{wrap_errorable_function, Error},
    missile_templates::load_missiles::{self, load_missiles},
    my_error, MainWindow,
};

pub fn on_open_missiles_view_handler(
    main_window_weak: Weak<MainWindow>,
    missiles_path: PathBuf,
) -> impl Fn() {
    move || {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || open_missiles_view(&missiles_path));
    }
}

#[instrument(skip())]
fn open_missiles_view(missiles_path: &PathBuf) -> Result<(), Error> {
    info!("Initialising missiles view");
    trace!("Creating window");

    let window = crate::MissileWindow::new()
        .map_err(|err| my_error!("Failed to create fleet editor window", err))
        .unwrap();

    let missiles = load_missiles(missiles_path)?;
    let missiles_model = Rc::new(VecModel::from(missiles));
    window.set_missiles(missiles_model.clone().into());

    debug!("Setting up callbacks");
    {
        let missiles_model = missiles_model.clone();
        window.on_viewing(move |idx| {
            missiles_model.set_vec(
                missiles_model
                    .iter()
                    .enumerate()
                    .map(|(m_idx, mut missile)| {
                        if m_idx as i32 != idx {
                            missile.selected = false;
                        }
                        missile
                    })
                    .collect::<Vec<_>>(),
            );
        });
    }
    window.on_reload_missiles(load_missiles::on_reload_missiles_handler(
        window.as_weak(),
        missiles_model.clone(),
        missiles_path.to_path_buf(),
    ));

    info!("Opening missiles window");
    window
        .show()
        .map_err(|err| my_error!("Could not show missiles window.", err))
        .unwrap();
    Ok(())
}
