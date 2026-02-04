use color_eyre::Result;

use crate::{load_app_config, load_fleets};

pub fn test_load_fleets() -> Result<()> {
    load_app_config()?;
    let _ = load_fleets::load_fleets();

    Ok(())
}
