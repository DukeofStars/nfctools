use std::{collections::HashMap, fs::File, rc::Rc};

use lazy_static::lazy_static;
use slint::{ComponentHandle, Model, ToSharedString, VecModel, Weak};
use tracing::trace;
use xmltree::Element;

use crate::{
    error::{wrap_errorable_function, Error},
    my_error, FleetData, FleetEditorWindow, MainWindow, ShipData,
};

pub mod liner_hull_config;

lazy_static! {
    static ref BULKER_SEGMENTS: HashMap<&'static str, &'static str> = HashMap::from([
        ("Bulk-0-Bow", "38e7a28f-1b06-4b73-98ee-f03d1d8a81fe"),
        ("Bulk-1-Bow", "29eb9c63-6c47-40f2-8f46-4ed4da8d3386"),
        ("Bulk-2-Bow", "c534a876-3f8a-4315-a194-5dda0f84c2b3"),
        ("Bulk-0-Core", "d4c9a66d-81e6-49ee-9b33-82d7a1522bbf"),
        ("Bulk-1-Core", "e2c11e02-b770-495e-a3c2-3dc998eac5a6"),
        ("Bulk-2-Core", "429f178e-e369-4f51-8054-2e01dd0abea1"),
        ("Bulk-0-Stern", "78d72a9a-893c-41c6-bddd-f198dfcf77ee"),
        ("Bulk-1-Stern", "2f2b451c-4776-405c-9914-cad4764f1072"),
        ("Bulk-2-Stern", "a8bf77b9-b7e3-4498-bf91-d3e777a7f688"),
        ("Container-0-Bow", "2d7c228c-cbd6-425e-9590-a2f8ae8d5915"),
        ("Container-1-Bow", "541cf476-4952-4234-a35a-5f1aa9089316"),
        ("Container-2-Bow", "bb034299-84c2-456f-b271-c91249cd4375"),
        ("Container-0-Core", "18a6bc15-58b0-479c-82c3-1722768f033d"),
        ("Container-1-Core", "09354e51-953c-451a-b415-3e3361812650"),
        ("Container-2-Core", "2c68a462-a143-4c89-aea0-df09d4786e92"),
        ("Container-0-Stern", "674e0528-3e0c-48e4-8e5e-d3a559869104"),
        ("Container-1-Stern", "2dbd82fe-d365-4367-aef5-9bb2d3528528"),
        ("Container-2-Stern", "aff1eba2-048e-4477-956b-574f4d468f1d"),
        ("Superstructure-0", "42d07c1a-156b-4057-aaca-7a2024751423"),
        ("Superstructure-1", "c9d04445-3558-46b4-b6fc-7dca8617d438"),
        ("Superstructure-2", "9ebcea74-e9c9-45b3-b616-e12e3f491024"),
        ("Superstructure-3", "59344a67-9e7b-43df-9f7c-505ad9a0ab87"),
    ]);
    static ref BRIDGE_MODELS: Vec<&'static str> = Vec::from_iter([
        "42d07c1a-156b-4057-aaca-7a2024751423",
        "c9d04445-3558-46b4-b6fc-7dca8617d438",
        "9ebcea74-e9c9-45b3-b616-e12e3f491024",
        "59344a67-9e7b-43df-9f7c-505ad9a0ab87",
    ]);
    static ref BULK_BOWS: Vec<&'static str> = Vec::from_iter([
        "38e7a28f-1b06-4b73-98ee-f03d1d8a81fe",
        "29eb9c63-6c47-40f2-8f46-4ed4da8d3386",
        "c534a876-3f8a-4315-a194-5dda0f84c2b3",
    ]);
    static ref BULK_CORES: Vec<&'static str> = Vec::from_iter([
        "d4c9a66d-81e6-49ee-9b33-82d7a1522bbf",
        "e2c11e02-b770-495e-a3c2-3dc998eac5a6",
        "429f178e-e369-4f51-8054-2e01dd0abea1",
    ]);
    static ref BULK_STERNS: Vec<&'static str> = Vec::from_iter([
        "78d72a9a-893c-41c6-bddd-f198dfcf77ee",
        "2f2b451c-4776-405c-9914-cad4764f1072",
        "a8bf77b9-b7e3-4498-bf91-d3e777a7f688",
    ]);
    static ref CONTAINER_BOWS: Vec<&'static str> = Vec::from_iter([
        "2d7c228c-cbd6-425e-9590-a2f8ae8d5915",
        "541cf476-4952-4234-a35a-5f1aa9089316",
        "bb034299-84c2-456f-b271-c91249cd4375",
    ]);
    static ref CONTAINER_CORES: Vec<&'static str> = Vec::from_iter([
        "18a6bc15-58b0-479c-82c3-1722768f033d",
        "09354e51-953c-451a-b415-3e3361812650",
        "2c68a462-a143-4c89-aea0-df09d4786e92",
    ]);
    static ref CONTAINER_STERNS: Vec<&'static str> = Vec::from_iter([
        "674e0528-3e0c-48e4-8e5e-d3a559869104",
        "2dbd82fe-d365-4367-aef5-9bb2d3528528",
        "aff1eba2-048e-4477-956b-574f4d468f1d",
    ]);
}

pub fn on_open_fleet_editor_handler(
    main_window_weak: Weak<MainWindow>,
    fleets_model: Rc<VecModel<FleetData>>,
) -> impl Fn() {
    move || {
        let main_window = main_window_weak.unwrap();
        let _ = wrap_errorable_function(&main_window, || {
            let main_window = main_window_weak.unwrap();
            let window = FleetEditorWindow::new()
                .map_err(|err| my_error!("Failed to create fleet editor window", err))
                .unwrap();

            let cur_idx = main_window.get_cur_fleet_idx();
            if cur_idx == -1 {
                return Err(my_error!("No fleet selected", ""));
            }
            let fleet = fleets_model.iter().nth(cur_idx as usize).ok_or(my_error!(
                "Selected fleet doesn't exist",
                "cur_fleet_idx points to a nonexistant fleet"
            ))?;

            window.set_fleet_name(fleet.name);

            let element = {
                trace!("Opening fleet file");
                let fleet_file = File::open(&fleet.path).map_err(|err| {
                    my_error!(
                        format!("Failed to open fleet '{}'", fleet.path.to_string()),
                        err
                    )
                })?;
                trace!("Parsing fleet file");
                Element::parse(fleet_file)
                    .map_err(|err| my_error!("Failed to parse fleet file", err))?
            };
            let ships_elem = element
                .get_child("Ships")
                .ok_or(my_error!("Failed to get ships list", "Fleet has no ships"))?;

            let ships = ships_elem
                .children
                .iter()
                .map(|ship_elem| {
                    let ship_elem = ship_elem
                        .as_element()
                        .ok_or(my_error!("Invalid fleet file", "Ship is not an element"))?;

                    let name = ship_elem
                        .get_child("Name")
                        .ok_or(my_error!("Invalid fleet file", "Ship has no name"))?
                        .get_text()
                        .ok_or(my_error!("Invalid fleet file", "Ship has no name"))?
                        .to_shared_string();
                    let hulltype = ship_elem
                        .get_child("HullType")
                        .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                        .get_text()
                        .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                        .to_shared_string();
                    let cost: i32 = ship_elem
                        .get_child("Cost")
                        .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                        .get_text()
                        .ok_or(my_error!("Invalid fleet file", "Ship has no HullType"))?
                        .parse()
                        .map_err(|err| {
                            my_error!(
                                "Invalid fleet file",
                                format!("Failed to parse cost: {}", err)
                            )
                        })?;

                    Ok(ShipData {
                        class: hulltype,
                        name,
                        cost,
                    })
                })
                .collect::<Result<Vec<ShipData>, Error>>()?;

            let ships_model = std::rc::Rc::new(slint::VecModel::from(ships));
            window.set_ships(ships_model.clone().into());

            window.on_save_liner_config(liner_hull_config::on_save_liner_config_handler(
                main_window.as_weak(),
                window.as_weak(),
                fleets_model.clone(),
            ));
            window.on_get_liner_config(liner_hull_config::on_get_liner_config_handler(
                main_window.as_weak(),
                window.as_weak(),
                fleets_model.clone(),
            ));
            window.on_load_dressings(liner_hull_config::on_load_dressings_handler(
                main_window.as_weak(),
                window.as_weak(),
            ));

            window
                .show()
                .map_err(|err| my_error!("Could not show fleet editor window.", err))
                .unwrap();
            Ok(())
        });
    }
}
