use lazy_static::lazy_static;

lazy_static! {
    pub static ref LN_BOW_DRESSINGS: [[Vec<&'static str>; 8]; 3] = [
        // Bow Dressings
        [
            vec!["None", "Top crates"],
            vec!["None", "Bottom crates"],
            vec!["None", "Flat arm bottom"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
        ],
        [
            vec!["None", "Tanks"],
            vec!["None", "Top crates"],
            vec!["None", "Flat arm top"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
        ],
        [
            vec!["None", "Flat arm top"],
            vec!["None", "Triple arm bottom", "Double arm bottom"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
        ],


    ];
    pub static ref LN_CORE_DRESSINGS: [[Vec<&'static str>; 8]; 3] = [
        [
            vec!["None", "Top crates", "UNKNOWN", "UNKNOWN"],
            vec!["None", "Bottom crates"],
            vec!["None", "Vertical arm top"],
            vec!["None", "Flat arm bottom"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
        ],
        [
            vec!["None", "Tanks", "UNKNOWN", "UNKNOWN"],
            vec!["None", "Top crates"],
            vec!["None", "Big tanks", "Big crates"],
            vec!["None", "Flat arm top"],
            vec!["None", "Vertical arm top", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],

        ],
        [
            vec![
                "None",
                "Tanks & crates bottom",
                "Crates bottom",
                "Crates under wings",
            ],
            vec!["None", "Crates above wings"],
            vec!["None", "Vertical arm top"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
            vec!["None", "UNKNOWN", "UNKNOWN", "UNKNOWN"],
        ],
    ];
}
