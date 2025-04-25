use std::path::PathBuf;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FleetData {
    pub path: PathBuf,
    pub short_path: PathBuf,
    pub selected: bool,
    pub name: String,
}
