use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hasher,
    path::{Path, PathBuf},
};

use glob::Pattern;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use tracing::{debug, info, instrument, trace, warn};

use crate::{error::Error, fleet_io::read_fleet, my_error, MissileData};

pub mod missiles_window;
mod load_missiles;
mod update_fleets;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UsedMissilesCache {
    fleets: HashMap<PathBuf, FleetsUsedMissiles>,
}

impl UsedMissilesCache {
    pub fn update(&mut self, missiles_dir: &PathBuf, excluded_patterns: &Vec<Pattern>) -> Result<(), Error> {
        debug!("Purging missile cache of deleted fleets");
        for path in self.fleets.keys().cloned().collect::<Vec<_>>() {
            if !path.exists() {
                trace!("Removing '{}' from the cache", path.display());
                self.fleets.remove(&path);
            }
        }
        debug!("Updating existing cache entries");
        self.recurse_fleets(missiles_dir, excluded_patterns)?;

        Ok(())
    }

    #[instrument(skip(excluded_patterns))]
    pub fn generate_from_fleets(
        missiles_dir: &PathBuf,
        excluded_patterns: &Vec<Pattern>,
    ) -> Result<UsedMissilesCache, Error> {
        info!("Generating fresh UsedMissilesCache");

        let mut missile_cache = UsedMissilesCache::default();

        missile_cache.recurse_fleets(missiles_dir, excluded_patterns)?;

        Ok(missile_cache)
    }

    fn recurse_fleets(&mut self, path: &PathBuf, excluded_patterns: &Vec<Pattern>) -> Result<(), Error> {
        trace!("Reading dir '{}'", path.display());
        let read_dir = std::fs::read_dir(path).map_err(|err| my_error!("Failed to read directory", err))?;

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
                        let file_bytes = std::fs::read(child.path())
                            .map_err(|err| my_error!("Failed to read bytes from fleet file", err))?;
                        hasher.write(&file_bytes);
                        hasher.finish()
                    };
                    if hash == old_fleet_data.hash {
                        trace!("Reusing cached missile data for '{}'", child.path().display());
                        continue;
                    }
                }
                match FleetsUsedMissiles::from_fleet_file(child.path()) {
                    Ok(used_missiles) => {
                        self.fleets.insert(child.path(), used_missiles);
                    }
                    Err(err) => {
                        warn!("Skipping invalid fleet '{}': {}", child.path().display(), err,);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FleetsUsedMissiles {
    name: String,
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
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
            serde::de::Error::invalid_value(serde::de::Unexpected::Other("invalid hex"), &"a valid hex string")
        })
    }
}

impl FleetsUsedMissiles {
    pub fn from_fleet_file(path: impl AsRef<Path>) -> Result<FleetsUsedMissiles, Error> {
        let path = path.as_ref();

        // Generate hash
        let hash = {
            let mut hasher = metrohash::MetroHash::new();
            let file_bytes =
                std::fs::read(path).map_err(|err| my_error!("Failed to read bytes from fleet file", err))?;
            hasher.write(&file_bytes);
            hasher.finish()
        };

        // Pulling missile templates
        debug!("Pulling used missile templates from '{}'", path.display());

        let fleet = read_fleet(path)?;

        let Some(missile_types_elem) = fleet.missile_types else {
            return Ok(FleetsUsedMissiles {
                name: fleet.name.clone(),
                hash,
                used_missiles: Vec::new(),
            });
        };

        let mut used_missiles = Vec::new();
        missile_types_elem.missile_template.map(|x| {
            x.iter()
                .for_each(|missile_template| used_missiles.push(MissileTemplateId::from_missile(&missile_template)))
        });

        Ok(FleetsUsedMissiles {
            name: fleet.name,
            hash,
            used_missiles,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MissileTemplateId {
    template_name: Option<String>,
    name: String,
}
impl Display for MissileTemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.template_name.as_ref().unwrap_or(&self.name))
    }
}
impl PartialEq for MissileTemplateId {
    fn eq(&self, other: &Self) -> bool {
        (self.template_name.is_some() && self.template_name == other.template_name) || self.name == other.name
    }
}
impl MissileTemplateId {
    pub fn from_missile(missile: &schemas::MissileTemplate) -> MissileTemplateId {
        MissileTemplateId {
            template_name: missile.associated_template_name.clone(),
            name: format!("{} {}", missile.designation, missile.nickname),
        }
    }

    pub fn from_missile_data(missile_data: &MissileData) -> MissileTemplateId {
        MissileTemplateId {
            template_name: if missile_data.template_name.as_str() == "" {
                None
            } else {
                Some(missile_data.template_name.to_string())
            },
            name: format!("{} {}", missile_data.designation, missile_data.nickname),
        }
    }
}
