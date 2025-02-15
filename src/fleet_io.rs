use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use schemas::{Fleet, MissileTemplate};
use tracing::trace;

use crate::{error::Error, my_error};

pub fn read_fleet(path: impl AsRef<Path>) -> Result<Fleet, Error> {
    let path = path.as_ref();

    trace!("Opening fleet '{}'", path.display());
    let file = BufReader::new(File::open(path).map_err(|err| my_error!("Failed to open fleet file", err))?);
    trace!("Parsing fleet '{}'", path.display());
    let fleet =
        quick_xml::de::from_reader(file).map_err(|err| my_error!("Failed to parse fleet file", err))?;
    Ok(fleet)
}

pub fn write_fleet(path: impl AsRef<Path>, fleet: &Fleet) -> Result<(), Error> {
    let path = path.as_ref();

    let _ = std::fs::remove_file(path);
    trace!("Creating fleet file '{}'", path.display());
    let file =
        BufWriter::new(File::create_new(path).map_err(|err| my_error!("Failed to create fleet file", err))?);
    trace!("Serializing fleet '{}'", path.display());
    quick_xml::se::to_utf8_io_writer(file, fleet)
        .map_err(|err| my_error!("Failed to serialize fleet file", err))?;
    Ok(())
}

pub fn read_missile(path: impl AsRef<Path>) -> Result<MissileTemplate, Error> {
    let path = path.as_ref();

    trace!("Opening missile '{}'", path.display());
    let file = BufReader::new(File::open(path).map_err(|err| my_error!("Failed to open missile file", err))?);

    trace!("Parsing missile '{}'", path.display());
    let missile =
        quick_xml::de::from_reader(file).map_err(|err| my_error!("Failed to parse missile file", err))?;
    Ok(missile)
}

#[allow(dead_code)]
pub fn write_missile(path: impl AsRef<Path>, missile: &MissileTemplate) -> Result<(), Error> {
    let path = path.as_ref();

    let _ = std::fs::remove_file(path);
    trace!("Creating missile '{}'", path.display());
    let file = BufWriter::new(
        File::create_new(path).map_err(|err| my_error!("Failed to create missile file", err))?,
    );
    trace!("Serializing missile '{}'", path.display());
    quick_xml::se::to_utf8_io_writer(file, missile)
        .map_err(|err| my_error!("Failed to serialize missile file", err))?;
    Ok(())
}
