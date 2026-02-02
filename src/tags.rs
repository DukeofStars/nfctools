use std::{collections::HashMap, fs::OpenOptions, io::Write};

use chumsky::prelude::*;
use color::{AlphaColor, Srgb};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use serde::{Deserialize, Serialize};
use text::whitespace;
use tracing::{debug, error, trace};

pub type Color = AlphaColor<Srgb>;

pub fn load_tags() -> Result<TagsRepository> {
    let tags_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(eyre!("OS not recognised?"))
        .wrap_err("Failed to retrieve config dir")?
        .preference_dir()
        .join("tags.toml");
    debug!("Loading tags from '{}'", tags_path.display());
    let tags_file = std::fs::read_to_string(&tags_path)
        .inspect_err(|_| {
            trace!("No tags file found, using default config values")
        })
        .unwrap_or_default();
    let tags_repo: TagsRepository =
        toml::from_str(&tags_file).wrap_err("Failed to parse tags file")?;

    Ok(tags_repo)
}

pub fn save_tags(tags_repo: &TagsRepository) -> Result<()> {
    let tags_path = directories::ProjectDirs::from("", "", "NebTools")
        .ok_or(eyre!("OS not recognised?"))
        .wrap_err("Failed to retrieve config dir")?
        .preference_dir()
        .join("tags.toml");
    debug!("Writing tags to '{}'", tags_path.display());
    let mut tags_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&tags_path)
        .wrap_err("Failed to open tags file")?;
    let toml =
        toml::to_string(tags_repo).wrap_err("Failed to serialize tags")?;
    tags_file
        .write_all(toml.as_bytes())
        .wrap_err("Failed to write tags file")?;

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagsRepository {
    tags: HashMap<String, Color>,
}

impl Drop for TagsRepository {
    fn drop(&mut self) {
        if let Err(err) = save_tags(&self) {
            error!("{}", err.wrap_err("Failed to save tags"));
        }
    }
}

impl TagsRepository {
    pub fn add_tag(&mut self, name: String, color: Color) {
        self.tags.insert(name, color);
    }
    pub fn get_tag(&self, name: &String) -> Option<&Color> {
        self.tags.get(name)
    }
}
impl Default for TagsRepository {
    fn default() -> Self {
        Self {
            tags: Default::default(),
        }
    }
}

pub fn get_tags_from_description(desc: &str) -> Result<(Vec<Tag>, String)> {
    if desc.starts_with("Tags:") {
        trace!("Parsing tags");
        let parser = tags_parser();
        let res = parser
            .parse(desc)
            .map_err(|mut errs| eyre!(errs.remove(0)))
            .wrap_err("Failed to parse tags")?;

        trace!("Found {} tags: {:?}", res.0.len(), res.0);

        Ok(res)
    } else {
        Ok((Vec::new(), desc.to_string()))
    }
}

fn tags_parser() -> impl Parser<char, (Vec<Tag>, String), Error = Simple<char>>
{
    just("Tags:")
        .ignore_then(whitespace())
        .ignore_then(
            just('<')
                .ignore_then(just("color="))
                .ignore_then(take_until(just('>')).map(|(hex, _)| {
                    // let full = u32::from_str_radix(&String::from_iter(hex), 16)
                    //     .unwrap();

                    <Color as std::str::FromStr>::from_str(&String::from_iter(
                        hex,
                    ))
                    .expect("Hex should already be guaranteed")
                }))
                .then(take_until(just("</color>")))
                .map(|(col, (text, _))| Tag {
                    color: col,
                    name: String::from_iter(text).into(),
                })
                .then_ignore(whitespace())
                .repeated(),
        )
        .then(take_until(end()).map(|(text, _)| String::from_iter(text)))
}
