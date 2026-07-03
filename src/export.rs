use serde::{Deserialize, Serialize};
use color_eyre::{Result, eyre::{Context, bail}};

use crate::fleet_edit::EditableHullParams;

const LN_CONFIG_PREFIX: &'static str = "LNCONFIG:";

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

fn ser_to_hex<T>(t: &T) -> Result<String> where T: Serialize {
    let mut s = String::new();
    for byte in postcard::to_stdvec(t)? {
        s.push_str(&format!("{:X}", byte))
    }
    Ok(s)
}
fn hex_to_ser<T>(s: &str) -> Result<T> where T: for<'a> Deserialize<'a> {
    let mut bytes = Vec::new();
    for i in 0..s.len() {
        let value = u8::from_str_radix(&s[i..=i], 16).wrap_err("Invalid hex code")?;
        bytes.push(value);
    }
    Ok(postcard::from_bytes(&bytes)?)
}