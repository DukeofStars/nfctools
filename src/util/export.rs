use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use serde::{Deserialize, Serialize};

use crate::{
    fleet_edit::EditableHullParams, ui::formations::FormationTemplate,
};

const LN_CONFIG_PREFIX: &'static str = "LNCONFIG:";
const FORMATION_PREFIX: &'static str = "FORM:";

pub fn export_hull_config(config: &EditableHullParams) -> Result<String> {
    let ser = ser_to_hex(config)?;
    Ok(format!("{LN_CONFIG_PREFIX}{ser}"))
}

pub fn import_hull_config(s: &str) -> Result<EditableHullParams> {
    let Some(hex) = s.strip_prefix(LN_CONFIG_PREFIX) else {
        bail!("String not a valid ln config");
    };
    hex_to_ser(hex)
}

pub fn export_formation(formation: &FormationTemplate) -> Result<String> {
    let ser = ser_to_hex(formation)?;
    Ok(format!("{FORMATION_PREFIX}{ser}"))
}

pub fn import_formation(s: &str) -> Result<FormationTemplate> {
    let Some(hex) = s.strip_prefix(FORMATION_PREFIX) else {
        bail!("String not a valid formation");
    };
    hex_to_ser(hex)
}

fn ser_to_hex<T>(t: &T) -> Result<String>
where
    T: Serialize,
{
    let mut s = String::new();
    for byte in postcard::to_stdvec(t)? {
        s.push_str(&format!("{:02X}", byte))
    }

    let mut out = String::new();

    let mut zerocount = 0;
    for char in s.chars() {
        if char == '0' {
            zerocount += 1;
        } else {
            if zerocount > 1 {
                out.push_str(&format!("[{zerocount}]"));
                zerocount = 0;
            } else if zerocount == 1 {
                out.push('0');
                zerocount = 0;
            }
            out.push(char);
        }
    }
    if zerocount > 1 {
        out.push_str(&format!("[{zerocount}]"));
    } else if zerocount == 1 {
        out.push('0');
    }

    Ok(out)
}
fn hex_to_ser<T>(s: &str) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let mut real = String::new();
    let mut iter = s.chars();
    while let Some(char) = iter.next() {
        if char == '[' {
            let mut zerocounts = String::new();
            while let Some(char) = iter.next() {
                if char == ']' {
                    break;
                } else {
                    zerocounts.push(char);
                }
            }
            let zerocount = usize::from_str_radix(&zerocounts, 10).unwrap();
            real.push_str(&"0".repeat(zerocount));
        } else {
            real.push(char);
        }
    }

    let mut bytes = Vec::new();
    for i in (0..real.len()).step_by(2) {
        let value = u8::from_str_radix(&real[i..=(i + 1)], 16)
            .wrap_err("Invalid hex code")?;
        bytes.push(value);
    }
    Ok(postcard::from_bytes(&bytes)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        export::{
            export_formation, export_hull_config, import_formation,
            import_hull_config,
        },
        fleet_edit::EditableHullParams,
        ui::formations::{FormationTemplate, Point3Serde},
    };

    #[test]
    fn ln_round_trip() {
        let params = EditableHullParams {
            bow_type: 1,
            core_type: 1,
            stern_type: 1,
            superstructure_loc: 1,
            superstructure_type: 1,
            bow_dressings: [1; 8],
            core_dressings: [1; 8],
        };
        let ser = export_hull_config(&params).unwrap();
        let des = import_hull_config(&ser).unwrap();

        assert_eq!(params, des);
    }

    #[test]
    fn form_round_trip() {
        let formation = FormationTemplate {
            escorts: vec![
                Point3Serde {
                    x: 100.0,
                    y: 250.0,
                    z: 100.0
                };
                4
            ],
        };

        let ser = export_formation(&formation).unwrap();
        let des = import_formation(&ser).unwrap();

        assert_eq!(formation, des);
    }
}
