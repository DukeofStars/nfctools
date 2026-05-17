use crate::fleet_data::FleetData;

#[derive(Debug, PartialEq)]
pub struct SearchFilters {
    contains: Vec<String>,
    tags: Vec<String>,
}
impl SearchFilters {
    pub fn matches(&self, fleet_data: &FleetData) -> bool {
        if self.contains.is_empty() && self.tags.is_empty() {
            return true;
        }
        let (tags, desc) = crate::tags::get_tags_from_description(
            fleet_data.description.as_str(),
        )
        .unwrap();

        (self.contains.is_empty()
            || self.contains.iter().any(|c| {
                fleet_data.name.to_lowercase().contains(c)
                    || fleet_data
                        .short_path
                        .display()
                        .to_string()
                        .to_lowercase()
                        .contains(c)
                    || desc.to_lowercase().contains(c)
            }))
            && (self.tags.is_empty()
                || self.tags.iter().any(|tag_match| {
                    tags.iter().any(|tag| &tag.name.to_lowercase() == tag_match)
                }))
    }
}

pub fn parse_search_text(text: String) -> SearchFilters {
    let words = text.split_whitespace();
    let mut tags = Vec::new();
    let contains = words
        .filter_map(|word| {
            if word.starts_with("tag:") {
                tags.push(word.strip_prefix("tag:").unwrap().to_lowercase());
                None
            } else {
                Some(word.to_lowercase())
            }
        })
        .collect::<Vec<String>>();

    SearchFilters { contains, tags }
}
