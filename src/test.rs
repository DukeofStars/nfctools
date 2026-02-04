use std::str::FromStr;

use color_eyre::Result;
use glob::Pattern;

use crate::{load_app_config, load_fleets};

pub fn test_load_fleets() -> Result<()> {
    let config = load_app_config()?;
    let _ = load_fleets::load_fleets(
        config.saves_dir,
        &config
            .excluded_dirs
            .into_iter()
            .filter_map(|x| Pattern::from_str(&x).ok())
            .collect::<Vec<_>>(),
    );

    Ok(())
}
