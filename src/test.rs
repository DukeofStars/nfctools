use tracing::{error, info};

use crate::load_fleets;

pub fn test_fleets() {
    match load_fleets::load_fleets(false) {
        Ok(_) => {
            info!("All fleets successfully loaded");
        }
        Err(err) => {
            error!(?err, "Fleet test failed");
        }
    }
}
