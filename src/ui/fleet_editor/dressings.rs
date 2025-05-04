use lazy_static::lazy_static;

lazy_static! {
    pub static ref LN_BOW_DRESSINGS: [[Vec<&'static str>; 5]; 3] = [
        // Bow Dressings
        [
            vec!["None", "Top crates"],
            vec!["None", "Bottom crates"],
            vec!["None", "Flat arm bottom"],
            vec!["None"],
            vec!["None"],
        ],
        [
            vec!["None", "Tanks"],
            vec!["None", "Top crates"],
            vec!["None", "Flat arm top"],
            vec!["None"],
            vec!["None"],
        ],
        [
            vec!["None", "Flat arm top"],
            vec!["None", "Triple arm bottom", "Double arm bottom"],
            vec!["None"],
            vec!["None"],
            vec!["None"],
        ],


    ];
    pub static ref LN_CORE_DRESSINGS: [[Vec<&'static str>; 5]; 3] = [
        [
            vec!["None", "Top crates"],
            vec!["None", "Bottom crates"],
            vec!["None", "Vertical arm top"],
            vec!["None", "Flat arm bottom"],
            vec!["None"],
        ],
        [
            vec!["None", "Tanks"],
            vec!["None", "Top crates"],
            vec!["None", "Big tanks", "Big crates"],
            vec!["None", "Flat arm top"],
            vec!["None", "Vertical arm top"],
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
            vec!["None"],
            vec!["None"],
        ],
    ];
}
