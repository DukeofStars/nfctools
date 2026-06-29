use std::cell::RefCell;

use dioxus::desktop::muda::{Menu, MenuItem, Submenu};

thread_local! {
    pub static MENUBARS: RefCell<Option<Menubars>> = const { RefCell::new(None) };
}

#[allow(unused)]
#[derive(Clone)]
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
    tools_merge: MenuItem,

    // Help
    help_menu: Submenu,
    help_open_log: MenuItem,
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

    pub fn enable_scramble(&self) {
        // Scramble unimplemented
        // self.tools_scramble.set_enabled(true);
    }

    pub fn disable_scramble(&self) {
        self.tools_scramble.set_enabled(false);
    }
}
