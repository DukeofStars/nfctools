use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use color_eyre::{eyre::Context, Result};
use schemas::{Fleet, MissileTemplate};
use tracing::{info, instrument, trace, warn};

use crate::fleet_data::FleetData;

pub fn read_fleet(path: impl AsRef<Path>) -> Result<Fleet> {
    let path = path.as_ref();

    trace!("Opening fleet '{}'", path.display());
    let file =
        BufReader::new(File::open(path).wrap_err("Failed to open fleet file")?);

    trace!("Parsing fleet '{}'", path.display());
    let fleet = match quick_xml::de::from_reader(file) {
        Ok(fleet) => fleet,
        Err(err) => {
            warn!("{}", err);
            Err(err).wrap_err("Failed to parse fleet file")?
        }
    };
    Ok(fleet)
}

#[allow(dead_code)]
pub fn write_fleet(path: impl AsRef<Path>, fleet: &Fleet) -> Result<()> {
    let path = path.as_ref();

    let _ = std::fs::remove_file(path);
    trace!("Creating fleet file '{}'", path.display());
    let file = BufWriter::new(
        File::create_new(path).wrap_err("Failed to create fleet file")?,
    );
    trace!("Serializing fleet '{}'", path.display());
    quick_xml::se::to_utf8_io_writer(file, fleet)
        .wrap_err("Failed to serialize fleet file")?;
    Ok(())
}

#[allow(dead_code)]
pub fn read_missile(path: impl AsRef<Path>) -> Result<MissileTemplate> {
    let path = path.as_ref();

    trace!("Opening missile '{}'", path.display());
    let file = BufReader::new(
        File::open(path).wrap_err("Failed to open missile file")?,
    );

    trace!("Parsing missile '{}'", path.display());
    let missile = match quick_xml::de::from_reader(file) {
        Ok(missile) => missile,
        Err(err) => {
            warn!("{}", err);
            Err(err).wrap_err("Failed to parse missile file")?
        }
    };
    Ok(missile)
}

#[allow(dead_code)]
pub fn write_missile(
    path: impl AsRef<Path>,
    missile: &MissileTemplate,
) -> Result<()> {
    let path = path.as_ref();

    let _ = std::fs::remove_file(path);
    trace!("Creating missile '{}'", path.display());
    let file = BufWriter::new(
        File::create_new(path).wrap_err("Failed to create missile file")?,
    );
    trace!("Serializing missile '{}'", path.display());
    quick_xml::se::to_utf8_io_writer(file, missile)
        .wrap_err("Failed to serialize missile file")?;
    Ok(())
}

#[instrument(skip(fleet_data, fleet))]
pub fn save_fleet_data(fleet_data: &FleetData, fleet: &Fleet) -> Result<()> {
    info!("Saving description for fleet '{}'", &fleet_data.name);

    write_fleet(&fleet_data.path, &fleet)?;

    info!("Fleet description saved successfully");

    Ok(())
}
