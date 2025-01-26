use std::rc::Rc;

use chumsky::prelude::*;
use slint::{Color, VecModel};
use text::whitespace;
use tracing::debug;

use crate::{error::Error, my_error, Tag};

pub fn on_add_tag_handler(tags: Rc<VecModel<Tag>>) -> impl Fn(Tag) {
    move |tag| {
        debug!("Adding tag {:?}", tag);
        tags.push(tag)
    }
}

pub fn on_remove_tag_handler(tags: Rc<VecModel<Tag>>) -> impl Fn(i32) {
    move |idx| {
        debug!("Removing tag {}", idx);
        tags.remove(idx as usize);
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
