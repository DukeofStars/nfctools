use std::rc::Rc;

use slint::{Model, VecModel};
use tracing::{debug, info, instrument};

use crate::{
    error::Error,
    fleet_io::{read_fleet, write_fleet},
    my_error, FleetData, MainWindow, Tag,
};

#[instrument(skip(main_window, fleets_model, description, tags_model))]
pub fn save_fleet_data(
    main_window: &MainWindow,
    fleets_model: Rc<VecModel<FleetData>>,
    tags_model: Rc<VecModel<Tag>>,
    description: String,
) -> Result<(), Error> {
    let cur_fleet_idx = main_window.get_cur_fleet_idx();
    if cur_fleet_idx == -1 {
        return Ok(());
    }
    info!("Saving description for fleet {}", cur_fleet_idx);
    let fleet_data = fleets_model
        .iter()
        .nth(cur_fleet_idx as usize)
        .ok_or(my_error!(
            "Selected fleet doesn't exist",
            "cur_fleet_idx points to a nonexistant fleet"
        ))?;

    debug!("Inserting tags into description");
    let description = format!(
        "Tags: {}\n{}",
        tags_model
            .iter()
            .map(|tag| format!(
                "<color=#{:02x}{:02x}{:02x}>{}</color>",
                tag.color.red(),
                tag.color.green(),
                tag.color.blue(),
                tag.name
            ))
            .collect::<Vec<_>>()
            .join(" "),
        description,
    );

    let mut fleet = read_fleet(&fleet_data.path)?;

    fleet.description = Some(description);

    write_fleet(&fleet_data.path, &fleet)?;

    info!("Fleet description saved successfully");

    Ok(())
}
