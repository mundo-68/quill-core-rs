// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::format_const::{TEXT_ATTR_BACK_GROUND, TEXT_ATTR_COLOR, TEXT_ATTR_FONT, TEXT_ATTR_SIZE};
use crate::util::lookup::{AttributesLookup, Attributor};
use anyhow::Result;
use delta::attributes::Attributes;
use dom::dom_element::DomElement;
/// Text attributes apply an HTML attribute to a given HTML element.
///
/// All these formats are:
///  1) a separate HTML element, embedded in a `<span>` or `<p>` element
///  2) form a linked chain of format html elements, ending with a TEXT element
///  3) Some attributes can be added to the format html element
///
/// Things supported here fit under: `<SPAN style="xxx">..</SPAN>`
///
use once_cell::sync::OnceCell;

// font: verdana, times, ...
// size: 6, 8
// color: red,green, ..., or rgb(128, 128, 0)
pub static TEXT_ATTRIBUTES: OnceCell<AttributesLookup> = OnceCell::new();
pub fn initialise() {
    if let Some(_attr) = TEXT_ATTRIBUTES.get() {
        return;
    }
    let mut attr = AttributesLookup::new(3);
    attr.fill_one(TEXT_ATTR_FONT, "ql-font-");
    attr.fill_one(TEXT_ATTR_SIZE, "font-size");
    attr.fill_one(TEXT_ATTR_COLOR, "color");
    attr.fill_one(TEXT_ATTR_BACK_GROUND, "background-color");
    TEXT_ATTRIBUTES
        .set(attr)
        .expect("did you call TEXT_ATTRIBUTES::initialise() twice?");
}

/// # apply_text_attributes()
///
/// Note that of all text formats the font is a class element.<br>
/// The other formats are style elements.
///
/// For that reason we single out the `ql-font-` value which is found for the font selection.
pub(crate) fn apply_text_attributes(element: &DomElement, attr: &Attributes) -> Result<()> {
    let classes = element.get_classes();
    for (format, attr_val) in Attributor::selected(attr, TEXT_ATTRIBUTES.get().unwrap()) {
        if attr_val.is_string() {
            if format == "ql-font-" {
                let k = [format, attr_val.str_val()?].concat();
                DomElement::add_class(&classes, &k);
            } else {
                DomElement::add_style(element, format, attr_val.str_val()?);
            }
        } else if format == "ql-font-" {
            DomElement::remove_class_starts_with(&classes, format);
        } else {
            DomElement::remove_style(element, format);
        }
    }
    Ok(())
}

pub(crate) fn has_text_attributes(attr: &Attributes) -> bool {
    let text_attr = Attributor::all_key(TEXT_ATTRIBUTES.get().unwrap());
    let applies = text_attr
        .filter(|&&key| attr.contains_key(key))
        .collect::<Vec<&&str>>();
    !applies.is_empty()
}
