use lazy_static::lazy_static;
use schemas::Ship;
use tracing::{debug, info};

lazy_static! {
    pub static ref BRIDGE_MODELS: Vec<&'static str> = Vec::from_iter([
        "42d07c1a-156b-4057-aaca-7a2024751423",
        "c9d04445-3558-46b4-b6fc-7dca8617d438",
        "9ebcea74-e9c9-45b3-b616-e12e3f491024",
        "59344a67-9e7b-43df-9f7c-505ad9a0ab87",
    ]);
    pub static ref BULK_BOWS: Vec<&'static str> = Vec::from_iter([
        "38e7a28f-1b06-4b73-98ee-f03d1d8a81fe",
        "29eb9c63-6c47-40f2-8f46-4ed4da8d3386",
        "c534a876-3f8a-4315-a194-5dda0f84c2b3",
    ]);
    pub static ref BULK_CORES: Vec<&'static str> = Vec::from_iter([
        "d4c9a66d-81e6-49ee-9b33-82d7a1522bbf",
        "e2c11e02-b770-495e-a3c2-3dc998eac5a6",
        "429f178e-e369-4f51-8054-2e01dd0abea1",
    ]);
    pub static ref BULK_STERNS: Vec<&'static str> = Vec::from_iter([
        "78d72a9a-893c-41c6-bddd-f198dfcf77ee",
        "2f2b451c-4776-405c-9914-cad4764f1072",
        "a8bf77b9-b7e3-4498-bf91-d3e777a7f688",
    ]);
    pub static ref CONTAINER_BOWS: Vec<&'static str> = Vec::from_iter([
        "2d7c228c-cbd6-425e-9590-a2f8ae8d5915",
        "541cf476-4952-4234-a35a-5f1aa9089316",
        "bb034299-84c2-456f-b271-c91249cd4375",
    ]);
    pub static ref CONTAINER_CORES: Vec<&'static str> = Vec::from_iter([
        "18a6bc15-58b0-479c-82c3-1722768f033d",
        "09354e51-953c-451a-b415-3e3361812650",
        "2c68a462-a143-4c89-aea0-df09d4786e92",
    ]);
    pub static ref CONTAINER_STERNS: Vec<&'static str> = Vec::from_iter([
        "674e0528-3e0c-48e4-8e5e-d3a559869104",
        "2dbd82fe-d365-4367-aef5-9bb2d3528528",
        "aff1eba2-048e-4477-956b-574f4d468f1d",
    ]);
}

#[derive(Clone, Debug)]
pub struct EditableHullParams {
    pub bow_type: usize,
    pub core_type: usize,
    pub stern_type: usize,

    pub bow_dressings: [u8; 8],
    pub core_dressings: [u8; 8],

    pub superstructure_loc: usize,
    pub superstructure_type: usize,
}
impl Default for EditableHullParams {
    fn default() -> Self {
        Self {
            bow_type: Default::default(),
            core_type: Default::default(),
            stern_type: Default::default(),
            bow_dressings: Default::default(),
            core_dressings: Default::default(),
            superstructure_loc: Default::default(),
            superstructure_type: Default::default(),
        }
    }
}
pub fn get_ln_editable_hull_params(ship: &Ship) -> Option<EditableHullParams> {
    info!(ship = %ship.name, "Loading liner hull configuration");

    let mut out = EditableHullParams::default();

    if let Some(hull_config) = &ship.hull_config {
        let mut segments =
            hull_config.primary_structure.segment_configuration.iter();

        let segment = segments.next()?;
        out.bow_type = BULK_BOWS
            .iter()
            .position(|x| *x == segment.key.as_str())
            .unwrap();
        out.bow_dressings = segment
            .dressing
            .int
            .as_ref()
            .map(|ints| {
                let mut vec = ints
                    .iter()
                    .map(|s| s.parse::<u8>().unwrap())
                    .collect::<Vec<u8>>();

                // Make sure vec is exactly 5 items long
                if vec.len() < 8 {
                    for _ in 0..(8 - vec.len()) {
                        vec.push(0);
                    }
                }

                vec
            })
            .unwrap_or(vec![0, 0, 0, 0, 0, 0, 0, 0])
            .try_into()
            .unwrap();

        let segment = segments.next()?;
        out.core_type = BULK_CORES
            .iter()
            .position(|x| *x == segment.key.as_str())
            .unwrap();
        out.core_dressings = segment
            .dressing
            .int
            .as_ref()
            .map(|ints| {
                let mut vec = ints
                    .iter()
                    .map(|s| s.parse::<u8>().unwrap())
                    .collect::<Vec<u8>>();

                // Make sure vec is exactly 8 items long
                if vec.len() < 8 {
                    for _ in 0..(8 - vec.len()) {
                        vec.push(0);
                    }
                }

                vec
            })
            .unwrap_or(vec![0, 0, 0, 0, 0, 0, 0, 0])
            .try_into()
            .unwrap();

        let segment = segments.next()?;
        out.stern_type = BULK_STERNS
            .iter()
            .position(|x| *x == segment.key.as_str())
            .unwrap();

        let superstructure_config =
            &hull_config.secondary_structure.secondary_structure_config;
        out.superstructure_type = BRIDGE_MODELS
            .iter()
            .position(|k| *k == superstructure_config.key.as_str())
            .expect("Invalid superstructure key");
        out.superstructure_loc = superstructure_config
            .segment
            .parse()
            .expect("Invalid segment element");
    }

    debug!("Got liner config: {:?}", out);

    Some(out)
}

pub fn set_ln_hull_config(
    ship: &mut Ship,
    hull_params: EditableHullParams,
) -> Option<()> {
    info!(config = ?hull_params, "Saving ship hull configuration");

    let EditableHullParams {
        bow_type,
        core_type,
        stern_type,
        bow_dressings,
        core_dressings,
        superstructure_loc,
        superstructure_type,
    } = hull_params;

    set_bridge_type_and_anchor(ship, superstructure_type, superstructure_loc);

    set_ln_bow_type(ship, bow_type);
    set_ln_core_type(ship, core_type);
    set_ln_stern_type(ship, stern_type);

    let hull_config = ship.hull_config.as_mut()?;
    let primary_hull_config = &mut hull_config.primary_structure;

    let mut segment_configs =
        primary_hull_config.segment_configuration.iter_mut();
    if let Some(bow_config) = segment_configs.next() {
        let mut dressings = bow_dressings
            .iter()
            .rev()
            .skip_while(|x| **x == 0)
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        dressings.reverse();
        bow_config.dressing.int = Some(dressings)
    }
    if let Some(core_config) = segment_configs.next() {
        let mut dressings = core_dressings
            .iter()
            .rev()
            .skip_while(|x| **x == 0)
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        dressings.reverse();
        core_config.dressing.int = Some(dressings)
    }

    Some(())
}

pub fn set_ln_bow_type(ship: &mut Ship, segment_type: usize) -> Option<()> {
    let hex = BULK_BOWS[segment_type];

    let hull_config = ship.hull_config.as_mut()?;
    let bow_config = hull_config
        .primary_structure
        .segment_configuration
        .get_mut(0)?;

    bow_config.key = hex.to_string();

    Some(())
}
pub fn set_ln_core_type(ship: &mut Ship, segment_type: usize) -> Option<()> {
    let hex = BULK_CORES[segment_type];

    let hull_config = ship.hull_config.as_mut()?;
    let bow_config = hull_config
        .primary_structure
        .segment_configuration
        .get_mut(1)?;

    bow_config.key = hex.to_string();

    Some(())
}
pub fn set_ln_stern_type(ship: &mut Ship, segment_type: usize) -> Option<()> {
    let hex = BULK_STERNS[segment_type];

    let hull_config = ship.hull_config.as_mut()?;
    let bow_config = hull_config
        .primary_structure
        .segment_configuration
        .get_mut(2)?;

    bow_config.key = hex.to_string();

    Some(())
}

pub fn set_bridge_type_and_anchor(
    ship: &mut Ship,
    superstructure_type: usize,
    superstructure_segment: usize,
) -> Option<()> {
    let bridge_config = &mut ship
        .hull_config
        .as_mut()?
        .secondary_structure
        .secondary_structure_config;

    bridge_config.key = BRIDGE_MODELS[superstructure_type].to_string();
    bridge_config.segment = superstructure_segment.to_string();

    Some(())
}
