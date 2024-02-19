// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use delta::attributes::Attributes;
use delta::types::attr_val::AttrVal;
use std::collections::HashMap;
use std::slice::Iter;

/// # AttributesLookup 
/// 
/// The lookup module provides for:
///  1  vector which give a unique sequence of keys
///  2  map which gives a HTML tag or HTML attribute name for a given key
///
///  With as:
///  - Input: `Attributes<a_key,a_val>` --> user provided input
///  - Const: `MAP<a_key,html_key>`  --> provides translation
///  - Const: `VEC<a_key>`  --> provides a fixed sequence
///
/// Now we loop over `VEC<a_key>` and get pairs `(html_key, a_val)`

/// This stores the static lookup and attribute order
#[derive(Debug)]
pub struct AttributesLookup<'a> {
    pub order: Vec<&'a str>,
    map: HashMap<&'a str, &'a str>,
}

impl<'a> AttributesLookup<'a> {
    pub fn new(length: usize) -> Self {
        AttributesLookup {
            order: Vec::with_capacity(length),
            map: HashMap::new(),
        }
    }

    /// fill the structure in the right order; Here...
    pub fn fill_one(&mut self, attrib: &'a str, html_tag: &'a str) {
        self.order.push(attrib);
        self.map.insert(attrib, html_tag);
    }
}

/// This allows the user to loop over all attributes in a given order
pub struct Attributor<'a> {
    attr: &'a Attributes,
    lookup: &'a AttributesLookup<'a>,
    it: Iter<'a, &'a str>,
}

impl<'a> Attributor<'a> {
    /// Returns an iterator which gives the (HTML_KEY, HTML_VAL) back.
    /// We select over the keys in the attr input
    pub fn selected(attr: &'a Attributes, lookup: &'a AttributesLookup) -> Self {
        Attributor {
            attr,
            lookup,
            it: lookup.order.iter(),
        }
    }

    /// Returns an iterator over all keys
    pub fn all_key(lookup: &'a AttributesLookup) -> Iter<'a, &'a str> {
        lookup.order.iter()
    }
}

impl<'a> Iterator for Attributor<'a> {
    type Item = (&'a str, &'a AttrVal);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(&k) = self.it.next() {
                // will run out eventually rendering `None`
                if let Some(val) = self.attr.get(k) {
                    let html_key = self.lookup.map.get(k).unwrap();
                    return Some((html_key, val));
                }
                //continue up to next
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::OnceCell;

    static LOOKUP: OnceCell<AttributesLookup> = OnceCell::new();
    pub fn initialise() {
        if let Some(_attr) = LOOKUP.get() {
            return;
        }
        let mut attr = AttributesLookup::new(6);
        attr.fill_one("key_1", "html_tag_1");
        attr.fill_one("key_2", "html_tag_2");
        attr.fill_one("key_3", "html_tag_3");
        attr.fill_one("key_4", "html_tag_4");
        attr.fill_one("key_5", "html_tag_5");
        attr.fill_one("key_6", "html_tag_6");
        LOOKUP
            .set(attr)
            .expect("did you call LOOKUP::initialise() twice?");
    }

    fn do_something(format: &str) {
        println!("bool or null: {}", format)
    }

    #[test]
    fn test_lookup() {
        initialise();
        let mut attr = Attributes::default();
        attr.insert("key_1", "val_1");
        attr.insert("key_3", true);
        attr.insert("key_6", AttrVal::Null);

        for (format, attr_val) in Attributor::selected(&attr, LOOKUP.get().unwrap()) {
            if let AttrVal::Bool(b) = attr_val {
                if *b {
                    do_something(format);
                }
            }
            if let AttrVal::Null = attr_val {
                do_something(format);
            }
            println!("found: ({:?}, {:?})", format, attr_val);
        }

        for &i in Attributor::all_key(&LOOKUP.get().unwrap()) {
            println!("found 'all key': ({})", i);
        }
    }
}
