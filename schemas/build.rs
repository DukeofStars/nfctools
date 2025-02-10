use std::{fs::read_dir, path::Path};

use quick_xml::Reader;
use xml_schema_generator::{extend_struct, into_struct, Element, Options};

fn main() {
    let fleets_folder = r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets"#;

    let fleet1 = std::fs::read_to_string("./assets/ExampleFleet1.fleet")
        .expect("Failed to read example fleet 1");
    let mut root = match into_struct(&mut Reader::from_str(&fleet1)) {
        Ok(root) => root,
        Err(_) => panic!("expected to successfully parse into struct"),
    };
    root = generate_fleet_schema_from_path(fleets_folder, root);
    let struct_as_string = "// This file was automatically generated. Do not modify.\nuse serde::{Deserialize, Serialize};\n\n".to_owned()
+ &root.to_serde_struct(&Options::quick_xml_de().derive("Serialize, Deserialize, Clone"));

    std::fs::write("./src/lib.rs", struct_as_string).expect("Failed to write fleet schema");

    // let missile_folder = r#"C:\Program Files (x86)\Steam\steamapps\common\Nebulous\Saves\Fleets"#;
    // let missile1 = std::fs::read_to_string("./assets/ExampleMissile1.fleet")
    //     .expect("Failed to read example fleet 1");
    // let mut root = match into_struct(&mut Reader::from_str(&missile1)) {
    //     Ok(root) => root,
    //     Err(_) => panic!("expected to successfully parse into struct"),
    // };
    // root = generate_missile_schema_from_path(missile_folder, root);
    // let struct_as_string = "// This file was automatically generated. Do not modify.\nuse serde::{Deserialize, Serialize};\n\n".to_owned()
    //     + &root.to_serde_struct(&Options::quick_xml_de().derive("Serialize, Deserialize, Clone"));

    // std::fs::write("./src/missile.rs", struct_as_string).expect("Failed to write missile schema");
}

fn generate_fleet_schema_from_path(
    path: impl AsRef<Path>,
    mut root: Element<String>,
) -> Element<String> {
    for child in read_dir(path).unwrap() {
        let child = child.unwrap();

        if child.file_type().unwrap().is_dir() {
            root = generate_fleet_schema_from_path(&child.path(), root);
        } else if child.file_type().unwrap().is_file() {
            if child.path().extension().map(|s| s.to_str()) == Some(Some("fleet")) {
                eprintln!("Extending struct with '{}'", child.path().display());
                root = match extend_struct(
                    &mut Reader::from_str(std::fs::read_to_string(child.path()).unwrap().as_str()),
                    root.clone(),
                ) {
                    Ok(root) => root,
                    Err(_) => root,
                };
            }
        }
    }

    root
}

// fn generate_missile_schema_from_path(
//     path: impl AsRef<Path>,
//     mut root: Element<String>,
// ) -> Element<String> {
//     for child in read_dir(path).unwrap() {
//         let child = child.unwrap();

//         if child.file_type().unwrap().is_dir() {
//             root = generate_fleet_schema_from_path(&child.path(), root);
//         } else if child.file_type().unwrap().is_file() {
//             if child.path().extension().map(|s| s.to_str()) == Some(Some("missile")) {
//                 root = match extend_struct(
//                     &mut Reader::from_str(std::fs::read_to_string(child.path()).unwrap().as_str()),
//                     root.clone(),
//                 ) {
//                     Ok(root) => root,
//                     Err(_) => root,
//                 };
//             }
//         }
//     }

//     root
// }
