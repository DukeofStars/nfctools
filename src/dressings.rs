use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    /// Map of LN dressings, (segment key, dressing slot) -> dressing idx -> dressing title
    pub static ref LN_DRESSINGS: HashMap<(&'static str, usize), Vec<&'static str>> = HashMap::from_iter([
        // Bow A
        (("38e7a28f-1b06-4b73-98ee-f03d1d8a81fe", 0), vec!["None", "Tanks"]),
        (("38e7a28f-1b06-4b73-98ee-f03d1d8a81fe", 1), vec!["None", "Crates"]),
        (("38e7a28f-1b06-4b73-98ee-f03d1d8a81fe", 2), vec!["None"]),
        (("38e7a28f-1b06-4b73-98ee-f03d1d8a81fe", 3), vec!["None"]),
        // Bow B
        (("29eb9c63-6c47-40f2-8f46-4ed4da8d3386", 0), vec!["None"]),
        (("29eb9c63-6c47-40f2-8f46-4ed4da8d3386", 1), vec!["None"]),
        (("29eb9c63-6c47-40f2-8f46-4ed4da8d3386", 2), vec!["None"]),
        (("29eb9c63-6c47-40f2-8f46-4ed4da8d3386", 3), vec!["None"]),
        // Bow C
        (("c534a876-3f8a-4315-a194-5dda0f84c2b3", 0), vec!["None"]),
        (("c534a876-3f8a-4315-a194-5dda0f84c2b3", 1), vec!["None"]),
        (("c534a876-3f8a-4315-a194-5dda0f84c2b3", 2), vec!["None"]),
        (("c534a876-3f8a-4315-a194-5dda0f84c2b3", 3), vec!["None"]),
        // Core A
        (("d4c9a66d-81e6-49ee-9b33-82d7a1522bbf", 0), vec!["None", "Tanks"]),
        (("d4c9a66d-81e6-49ee-9b33-82d7a1522bbf", 1), vec!["None", "Crates"]),
        (("d4c9a66d-81e6-49ee-9b33-82d7a1522bbf", 2), vec!["None"]),
        (("d4c9a66d-81e6-49ee-9b33-82d7a1522bbf", 3), vec!["None"]),
        // Core B
        (("e2c11e02-b770-495e-a3c2-3dc998eac5a6", 0), vec!["None"]),
        (("e2c11e02-b770-495e-a3c2-3dc998eac5a6", 1), vec!["None"]),
        (("e2c11e02-b770-495e-a3c2-3dc998eac5a6", 2), vec!["None"]),
        (("e2c11e02-b770-495e-a3c2-3dc998eac5a6", 3), vec!["None"]),
        // Core C
        (("429f178e-e369-4f51-8054-2e01dd0abea1", 0), vec!["None"]),
        (("429f178e-e369-4f51-8054-2e01dd0abea1", 1), vec!["None"]),
        (("429f178e-e369-4f51-8054-2e01dd0abea1", 2), vec!["None"]),
        (("429f178e-e369-4f51-8054-2e01dd0abea1", 3), vec!["None"]),
        // // Stern A
        // (("78d72a9a-893c-41c6-bddd-f198dfcf77ee", 0), vec!["None", "Tanks"]),
        // (("78d72a9a-893c-41c6-bddd-f198dfcf77ee", 1), vec!["None", "Crates"]),
        // (("78d72a9a-893c-41c6-bddd-f198dfcf77ee", 2), vec!["None"]),
        // (("78d72a9a-893c-41c6-bddd-f198dfcf77ee", 3), vec!["None"]),
        // // Stern B
        // (("2f2b451c-4776-405c-9914-cad4764f1072", 0), vec!["None"]),
        // (("2f2b451c-4776-405c-9914-cad4764f1072", 1), vec!["None"]),
        // (("2f2b451c-4776-405c-9914-cad4764f1072", 2), vec!["None"]),
        // (("2f2b451c-4776-405c-9914-cad4764f1072", 3), vec!["None"]),
        // // Stern C
        // (("a8bf77b9-b7e3-4498-bf91-d3e777a7f688", 0), vec!["None"]),
        // (("a8bf77b9-b7e3-4498-bf91-d3e777a7f688", 1), vec!["None"]),
        // (("a8bf77b9-b7e3-4498-bf91-d3e777a7f688", 2), vec!["None"]),
        // (("a8bf77b9-b7e3-4498-bf91-d3e777a7f688", 3), vec!["None"]),
    ]);
}
