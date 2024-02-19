// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::constants::DOCUMENT;
use crate::dom_element::DomElement;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node, Text};

pub static TEXT_TAG: &str = "#text";
//
// static SPACE: &'static str = &*"&nbsp;";
// static NEW_LINE: &'static str = &*"BR";

#[derive(Debug, Clone)]
pub struct DomText {
    text: Text,
}

impl From<Node> for DomText {
    fn from(n: Node) -> Self {
        DomText {
            text: n.dyn_into::<Text>().unwrap(),
        }
    }
}

//fixme yuk, but is seems to work
impl From<&Node> for DomText {
    fn from(n: &Node) -> Self {
        let p = n.parent_node().unwrap();
        let children = p.child_nodes();
        for i in 0..children.length() {
            let c = children.get(i);
            if n.is_same_node(c.as_ref()) {
                return DomText {
                    text: c.unwrap().dyn_into::<Text>().unwrap(),
                };
            }
        }
        panic!("From(&Node): detected node without parent??");
    }
}

impl From<Text> for DomText {
    fn from(t: Text) -> Self {
        DomText { text: t }
    }
}

impl DomText {
    pub fn new(value: &str) -> Self {
        DomText {
            text: DOCUMENT.with(|d| d.create_text_node(value)),
        }
    }

    pub fn node(&self) -> &Node {
        &self.text
    }

    pub fn text(&self) -> &Text {
        &self.text
    }

    // //FIXME: This should go in the transducers::text::xxx
    //
    // pub fn line_break_node_new() -> Element {
    //     DOCUMENT.with(|d| d.create_element(NEW_LINE).unwrap())
    // }

    pub fn set_text(&self, value: &str) {
        // http://jsperf.com/textnode-performance
        self.text.set_data(value);
    }

    pub fn get_text(&self) -> String {
        self.text.data()
    }

    pub fn insert_text(&self, offset: usize, value: &str) {
        let os = offset as u32;
        self.text
            .insert_data(os, value)
            .expect("DomText:insert_text()");
    }

    pub fn delete_text(&self, offset: usize, len: usize) {
        let l = len as u32;
        let o = offset as u32;
        self.text.delete_data(o, l).expect("DomText:delete_text()");
    }

    pub fn append_text(&self, value: &str) {
        self.text.append_data(value).expect("DomText:append_text()");
    }

    pub fn text_length(&self) -> usize {
        let l = self.text.length();
        l as usize
    }

    #[inline(always)]
    pub fn get_parent(&self) -> Option<Element> {
        self.text.parent_element()
    }

    //Removes the "self" from the parent that contains it
    #[inline(always)]
    pub fn rm_child_from_parent(&self) {
        let parent_o = self.get_parent();
        if let Some(parent) = parent_o {
            parent
                .remove_child(self.node())
                .expect("DomText:rm_child_from_parent()");
        }
    }
}

//Find first matching child
//searching depth first, but stopping at FIRST match.
#[inline(always)]
pub fn find_dom_text(node: &Node) -> Option<DomText> {
    if node.node_name() == TEXT_TAG {
        return Some(DomText::from(node));
    }
    let el = node.dyn_ref::<Element>().unwrap();
    let children = el.child_nodes();
    for i in 0..children.length() {
        let c = children.get(i).unwrap();
        if c.node_name() == TEXT_TAG {
            return Some(DomText::from(c));
        }
        let el: Element = c.dyn_into::<Element>().unwrap();
        let dom_el = DomElement::from(el);
        let res = find_dom_text(dom_el.node());
        if let Some(tn) = res {
            return Some(tn);
        }
    }
    None
}
