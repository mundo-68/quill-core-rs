// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::util::lookup::{AttributesLookup, Attributor};
use anyhow::Result;
use delta::attributes::Attributes;
use dom::dom_element::DomElement;
use once_cell::sync::OnceCell;

pub static BLOCK_FORMAT: OnceCell<AttributesLookup> = OnceCell::new();

pub fn initialise() {
    if let Some(_attr) = BLOCK_FORMAT.get() {
        return;
    }
    let mut attr = AttributesLookup::new(3);
    attr.fill_one("direction", "ql-direction-");
    attr.fill_one("align", "ql-align-");
    attr.fill_one("indent", "ql-indent-");

    BLOCK_FORMAT
        .set(attr)
        .expect("did you call BLOCK_FORMAT::initialise() twice?");
}

/// The attributes are a map: `key -> val`, where the `key` match the `key` in `BLOCK_FORMAT`
pub fn apply(element: &DomElement, attr: &Attributes) -> Result<()> {
    let classes = element.get_classes();
    for (format, attr_val) in Attributor::selected(attr, BLOCK_FORMAT.get().unwrap()) {
        if !attr_val.is_null() {
            let k = if attr_val.is_string() {
                [format, attr_val.str_val()?].concat()
            } else {
                [format.to_string(), attr_val.number_val()?.to_string()].concat()
            };
            DomElement::add_class(&classes, &k);
        } else {
            DomElement::remove_class_starts_with(&classes, format);
        }
    }
    Ok(())
}
