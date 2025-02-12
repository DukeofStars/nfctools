use std::{cell::RefCell, collections::HashMap, rc::Rc};

use chumsky::prelude::*;
use serde::{Deserialize, Serialize};
use slint::{Color, SharedString, VecModel, Weak};
use text::whitespace;
use tracing::{debug, trace};

use crate::{error::Error, my_error, MainWindow, Tag};

#[derive(Serialize, Deserialize, Debug)]
pub struct TagsRepository {
    tags: HashMap<String, Color>,
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

pub fn on_add_tag_handler(tags: Rc<VecModel<Tag>>, tags_repo: Rc<RefCell<TagsRepository>>) -> impl Fn(Tag) {
    move |tag| {
        debug!("Adding tag {:?}", tag);
        tags_repo
            .borrow_mut()
            .add_tag(tag.name.to_string(), tag.color.clone());
        tags.push(tag);
    }
}

pub fn on_remove_tag_handler(tags: Rc<VecModel<Tag>>) -> impl Fn(i32) {
    move |idx| {
        debug!("Removing tag {}", idx);
        tags.remove(idx as usize);
    }
}

pub fn on_lookup_tag_handler(
    main_window_weak: Weak<MainWindow>,
    tags_repo: Rc<RefCell<TagsRepository>>,
) -> impl Fn(SharedString) {
    move |name| {
        trace!("Looking up tag '{}'", &name);
        if let Some(color) = tags_repo.borrow().get_tag(&name.to_string()) {
            trace!("Tag '{}' found: {:?}", &name, color);
            let main_window = main_window_weak.unwrap();
            main_window.invoke_set_tag_color(color.clone());
        }
    }
}

pub fn get_tags_from_description(desc: &str) -> Result<(Vec<Tag>, String), Error> {
    if desc.starts_with("Tags:") {
        debug!("Parsing tags");
        let parser = tags_parser();
        let res = parser.parse(desc).map_err(|errs| {
            let err = errs.first().unwrap();
            my_error!("Failed to parse tags", err.clone().map(|e| e.to_string()))
        })?;

        debug!("Found {} tags: {:?}", res.0.len(), res.0);

        Ok(res)
    } else {
        Ok((Vec::new(), desc.to_string()))
    }
}

fn tags_parser() -> impl Parser<char, (Vec<Tag>, String), Error = Simple<char>> {
    just("Tags:")
        .ignore_then(whitespace())
        .ignore_then(
            just('<')
                .ignore_then(just("color=#"))
                .ignore_then(take_until(just('>')).map(|(hex, _)| {
                    let full = u32::from_str_radix(&String::from_iter(hex), 16).unwrap();
                    Color::from_argb_encoded(full).with_alpha(1.0)
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
