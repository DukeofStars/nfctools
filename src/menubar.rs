use dioxus::{
    desktop::muda::{accelerator::Accelerator, Menu, MenuItem, Submenu},
    html::{Code, Modifiers},
};

#[allow(unused)]
pub struct Menubars {
    // Fleets
    fleets_menu: Submenu,
    fleets_reload: MenuItem,

    // Edit
    edit_preferences: MenuItem,
    edit_menu: Submenu,

    // Tools
    tools_menu: Submenu,
    tools_scramble: MenuItem,
    tools_winpred: MenuItem,
}

impl Menubars {
    pub fn new() -> Self {
        let fleets_menu = Submenu::new("Fleets", true);
        let fleets_reload = MenuItem::with_id(
            "fleets-reload",
            "Reload Fleets",
            true,
            Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyR)),
        );
        fleets_menu.append_items(&[&fleets_reload]).unwrap();

        let edit_menu = Submenu::new("Edit", true);
        let edit_preferences =
            MenuItem::with_id("edit-preferences", "Preferences", true, None);
        edit_menu.append_items(&[&edit_preferences]).unwrap();

        let tools_menu = Submenu::new("Tools", true);
        let tools_scramble =
            MenuItem::with_id("tools-scramble", "Scramble Fleet", false, None);
        let tools_winpred =
            MenuItem::with_id("tools-winpred", "Win Predictor", false, None);
        tools_menu
            .append_items(&[&tools_scramble, &tools_winpred])
            .unwrap();

        Menubars {
            fleets_menu,
            fleets_reload,
            edit_menu,
            edit_preferences,
            tools_menu,
            tools_scramble,
            tools_winpred,
        }
    }

    pub fn attach_to_menu(&self, menu: &Menu) {
        menu.append_items(&[
            &self.fleets_menu,
            &self.edit_menu,
            &self.tools_menu,
        ])
        .unwrap();
    }
}
