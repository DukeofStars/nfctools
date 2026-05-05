use std::{collections::HashMap, fmt::Display, fs::OpenOptions, io::Write, ops::Deref, str::FromStr, sync::{Mutex, OnceLock}};

use chumsky::prelude::*;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use palette::{encoding::Srgb, rgb::Rgb};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use text::whitespace;
use tracing::{debug, error, trace, warn};

pub static TAGS_REPO: OnceLock<Mutex<TagsRepository>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
pub struct Color(pub Rgb<Srgb, u8>);
impl Deref for Color {
    type Target = Rgb<Srgb, u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", &self.0)
    }
}
impl FromStr for Color {
    type Err = palette::rgb::FromHexError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Rgb::<Srgb, u8>::from_str(s).map(|c| Color(c))
    }
}

pub fn init_tags() {
    let tags_repo = load_tags();
    match tags_repo {
        Ok(tags_repo) => TAGS_REPO.set(Mutex::new(tags_repo)).expect("init_tags called more than once"),
        Err(err) => {
            warn!(?err, "Failed to load tag repository file");
            TAGS_REPO.set(Mutex::new(TagsRepository::default())).expect("init_tags called more than once")
        },
    }
}

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

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub color: Color,
}


#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct TagsRepository {
    #[serde_as(as = "HashMap<_, DisplayFromStr>")]
    pub tags: HashMap<String, Color>,
}

impl TagsRepository {
    pub fn add_tag(&mut self, name: String, color: Color) {
        self.tags.insert(name, color);
        self.save();
    }
    pub fn get_tag(&self, name: &String) -> Option<&Color> {
        self.tags.get(name)
    }

    pub fn save(&self) {
        if let Err(err) = save_tags(&self) {
            error!("{}", err.wrap_err("Failed to save tags"));
        }
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
