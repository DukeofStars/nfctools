use std::{
    collections::HashMap,
    fs::File,
    hash::Hasher,
    path::{Path, PathBuf},
};

use glob::Pattern;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use tracing::{debug, info, trace, warn};
use xmltree::Element;

use crate::{error::Error, my_error};

pub mod load_missiles;
pub mod missiles_window;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UsedMissilesCache {
    fleets: HashMap<PathBuf, FleetsUsedMissiles>,
}

impl UsedMissilesCache {
    pub fn update(
        &mut self,
        saves_dir: &PathBuf,
        excluded_patterns: &Vec<Pattern>,
    ) -> Result<(), Error> {
        self.recurse_fleets(saves_dir, excluded_patterns)?;

        Ok(())
    }

    pub fn generate_from_fleets(
        saves_dir: &PathBuf,
        excluded_patterns: &Vec<Pattern>,
    ) -> Result<UsedMissilesCache, Error> {
        info!("Generating fresh UsedMissilesCache");

        let mut missile_cache = UsedMissilesCache::default();

        missile_cache.recurse_fleets(saves_dir, excluded_patterns)?;

        Ok(missile_cache)
    }

    fn recurse_fleets(
        &mut self,
        path: &PathBuf,
        excluded_patterns: &Vec<Pattern>,
    ) -> Result<(), Error> {
        trace!("Reading dir '{}'", path.display());
        let read_dir =
            std::fs::read_dir(path).map_err(|err| my_error!("Failed to read directory", err))?;

        'child_loop: for child in read_dir {
            let Ok(child) = child else {
                warn!("Skipping child");
                continue;
            };

            for pattern in excluded_patterns {
                if pattern.matches_path(child.path().as_path()) {
                    continue 'child_loop;
                }
            }

            let file_type = child
                .file_type()
                .map_err(|err| my_error!("Failed to get child file_type", err))?;
            if file_type.is_dir() {
                self.recurse_fleets(&child.path(), excluded_patterns)?;
            } else if file_type.is_file() {
                if child.path().extension().map(|s| s.to_str()) != Some(Some("fleet".into())) {
                    continue;
                }
                if let Some(old_fleet_data) = self.fleets.get(&child.path()) {
                    let hash = {
                        let mut hasher = metrohash::MetroHash::new();
                        let file_bytes = std::fs::read(child.path()).map_err(|err| {
                            my_error!("Failed to read bytes from fleet file", err)
                        })?;
                        hasher.write(&file_bytes);
                        hasher.finish()
                    };
                    if hash == old_fleet_data.hash {
                        trace!("Skipping '{}'", child.path().display());
                        continue;
                    }
                }
                self.fleets.insert(
                    child.path(),
                    FleetsUsedMissiles::from_fleet_file(child.path())?,
                );
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FleetsUsedMissiles {
    #[serde(
        serialize_with = "serialize_hash",
        deserialize_with = "deserialize_hash"
    )]
    hash: u64,
    used_missiles: Vec<MissileTemplateId>,
}

fn serialize_hash<S>(hash: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{:x}", hash))
}
fn deserialize_hash<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(HexToU64Visitor)
}
struct HexToU64Visitor;
impl<'de> Visitor<'de> for HexToU64Visitor {
    type Value = u64;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a u64 encoded in hex")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        u64::from_str_radix(s, 16).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Other("invalid hex"),
                &"a valid hex string",
            )
        })
    }
}

impl FleetsUsedMissiles {
    pub fn from_fleet_file(path: impl AsRef<Path>) -> Result<FleetsUsedMissiles, Error> {
        let path = path.as_ref();

        // Generate hash
        let hash = {
            let mut hasher = metrohash::MetroHash::new();
            let file_bytes = std::fs::read(path)
                .map_err(|err| my_error!("Failed to read bytes from fleet file", err))?;
            hasher.write(&file_bytes);
            hasher.finish()
        };

        // Pulling missile templates
        debug!("Pulling used missile templates from '{}'", path.display());

        let element = {
            trace!("Opening fleet file");
            let fleet_file = File::open(&path).map_err(|err| {
                my_error!(format!("Failed to open fleet '{}'", path.display()), err)
            })?;
            trace!("Parsing fleet file");
            Element::parse(fleet_file)
                .map_err(|err| my_error!("Failed to parse fleet file", err))?
        };

        let Some(missile_types_elem) = element.get_child("MissileTypes") else {
            return Ok(FleetsUsedMissiles {
                hash,
                used_missiles: Vec::new(),
            });
        };

        let mut used_missiles = Vec::new();
        for missile_type_elem in &missile_types_elem.children {
            if let Some(associated_missile_template_elem) = missile_type_elem
                .as_element()
                .map(|elem| elem.get_child("AssociatedTemplateName"))
                .flatten()
            {
                let Some(associated_missile_template_name) =
                    associated_missile_template_elem.get_text()
                else {
                    return Err(my_error!(
                        "Invalid fleet file",
                        "AssociatedTemplateName is not text"
                    ));
                };

                used_missiles.push(MissileTemplateId(
                    associated_missile_template_name.to_string(),
                ));
            }
        }

        Ok(FleetsUsedMissiles {
            hash,
            used_missiles,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MissileTemplateId(pub String);
