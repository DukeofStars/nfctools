use std::{path::PathBuf, rc::Rc};

use schemas::{Fleet, MissileTemplate, MissileTypes, Ship};
use slint::{Model, SharedString, VecModel, Weak};
use tracing::{info, instrument, trace};

use crate::{
    error::{wrap_errorable_function, Error},
    fleet_io::{read_fleet, write_fleet},
    my_error, FleetData, MainWindow,
};

pub fn on_merge_handler(
    main_window_weak: Weak<MainWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn(SharedString) {
    move |merge_output_name| {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            let selected_fleets = fleets_model.iter().filter(|f| f.selected).collect::<Vec<_>>();
            merge_fleets(merge_output_name.to_string().trim().to_string(), &selected_fleets)
        });
    }
}

#[instrument(skip(merge_output_name, selected_fleets))]
fn merge_fleets(merge_output_name: String, selected_fleets: &[FleetData]) -> Result<(), Error> {
    info!(
        "Merging fleets {:?} into '{}'",
        selected_fleets.iter().map(|f| &f.name).collect::<Vec<_>>(),
        merge_output_name
    );
    if merge_output_name == "" {
        return Err(my_error!(
            "No merge output name",
            "You must set an output name for the merged fleets"
        ));
    }

    let first_fleet = &selected_fleets[0];
    trace!("Primary fleet is '{}'", first_fleet.name);

    let fleets = selected_fleets
        .iter()
        .skip(1)
        .map(|fleet_data| read_fleet(&fleet_data.path))
        .collect::<Result<Vec<Fleet>, Error>>()?;

    let (ships_iter, missiles_iter) = fleets
        .into_iter()
        .map(|fleet| {
            (
                fleet.ships.map(|ships| ships.ship).flatten().unwrap_or_default(),
                fleet
                    .missile_types
                    .map(|m| m.missile_template)
                    .flatten()
                    .unwrap_or_default(),
            )
        })
        .unzip::<_, _, Vec<Vec<Ship>>, Vec<Vec<MissileTemplate>>>();
    let mut ships: Vec<Ship> = ships_iter.into_iter().flatten().collect();
    let mut missiles: Vec<MissileTemplate> = missiles_iter.into_iter().flatten().collect();

    let mut primary_fleet = read_fleet(&first_fleet.path)?;
    if let Some(ship) = primary_fleet
        .ships
        .as_mut()
        .map(|ships| ships.ship.as_mut())
        .flatten()
    {
        ship.append(&mut ships);
    }
    if let Some(missile_types) = primary_fleet.missile_types.as_mut() {
        if let Some(missile_template) = missile_types.missile_template.as_mut() {
            missile_template.append(&mut missiles);
        } else {
            missile_types.missile_template = Some(missiles)
        }
    } else {
        primary_fleet.missile_types = Some(MissileTypes {
            text: None,
            missile_template: Some(missiles),
        });
    }

    let output_path =
        PathBuf::from(r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets\"#)
            .join(&merge_output_name)
            .with_extension("fleet");

    write_fleet(&output_path, &primary_fleet)?;

    Ok(())
}
