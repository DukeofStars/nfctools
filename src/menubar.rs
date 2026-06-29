use std::cell::RefCell;

use dioxus::desktop::muda::{Menu, MenuItem, Submenu};

thread_local! {
    pub static MENUBARS: RefCell<Option<Menubars>> = const { RefCell::new(None) };
}

#[allow(unused)]
#[derive(Clone)]
pub struct Menubars {
    // Fleets
    pub fleets_menu: Submenu,
    pub fleets_reload: MenuItem,

    // Edit
    pub edit_preferences: MenuItem,
    pub edit_menu: Submenu,

    // Tools
    pub tools_menu: Submenu,
    pub tools_scramble: MenuItem,
    pub tools_winpred: MenuItem,
    pub tools_merge: MenuItem,

    // Help
    pub help_menu: Submenu,
    pub help_open_log: MenuItem,
}

impl Menubars {
    pub fn new() -> Self {
        let fleets_menu = Submenu::new("Fleets", true);
        let fleets_reload =
            MenuItem::with_id("fleets-reload", "Reload Fleets", true, None);
        fleets_menu.append_items(&[&fleets_reload]).unwrap();

        let edit_menu = Submenu::new("Edit", true);
        let edit_preferences =
            MenuItem::with_id("edit-preferences", "Preferences", true, None);
        edit_menu.append_items(&[&edit_preferences]).unwrap();

        let tools_menu = Submenu::new("Tools", true);
        let tools_scramble =
            MenuItem::with_id("tools-scramble", "Scramble Fleet", false, None);
        let tools_winpred =
            MenuItem::with_id("tools-winpred", "Win Predictor", true, None);
        let tools_merge = MenuItem::with_id("tools-merge", "Merge Fleets", true, None);
        tools_menu
            .append_items(&[&tools_scramble, &tools_winpred, &tools_merge])
            .unwrap();

        let help_menu = Submenu::new("Help", true);
        let help_open_log =
            MenuItem::with_id("help-open-log", "Open Log File", true, None);
        help_menu.append_items(&[&help_open_log]).unwrap();

        Menubars {
            fleets_menu,
            fleets_reload,
            edit_menu,
            edit_preferences,
            tools_menu,
            tools_scramble,
            tools_winpred,
            tools_merge,
            help_menu,
            help_open_log,
        }
    }

    pub fn attach_to_menu(&self, menu: &Menu) {
        menu.append_items(&[
            &self.fleets_menu,
            &self.edit_menu,
            &self.tools_menu,
            &self.help_menu,
        ])
        .unwrap();
    }
}
