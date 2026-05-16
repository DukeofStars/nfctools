use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Debug)]
pub struct FleetData {
    pub path: PathBuf,
    pub short_path: PathBuf,
    pub name: String,
    pub description: String,
}
