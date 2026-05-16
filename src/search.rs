use schemas::Fleet;

use crate::fleet_data::FleetData;

#[derive(Debug, PartialEq)]
pub struct SearchFilters {
    contains: String,
}
impl SearchFilters {
    pub fn matches(&self, fleet_data: &FleetData) -> bool {
        fleet_data.name.contains(&self.contains)
    }
}

pub fn parse_search_text(text: String) -> SearchFilters {
    SearchFilters { contains: text }
}