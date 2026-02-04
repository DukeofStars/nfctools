use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FleetData {
    pub path: PathBuf,
    pub short_path: PathBuf,
    pub selected: bool,
    pub name: String,
}
