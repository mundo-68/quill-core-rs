// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::constants::DOCUMENT;
use std::option::Option;
use wasm_bindgen::JsCast;
use web_sys::{DomTokenList, Element, HtmlElement, Node, NodeList};

#[derive(PartialEq, Debug, Clone)]
pub struct DomElement {
    element: Element,
}

impl From<Element> for DomElement {
    fn from(el: Element) -> Self {
        DomElement { element: el }
    }
}

impl DomElement {
    pub fn new(name: &str) -> Self {
        DomElement {
            element: DOCUMENT.with(|d| d.create_element(name).unwrap()),
        }
    }

    pub fn node_name(&self) -> String {
        self.element.node_name()
    }

    pub fn node(&self) -> &Node {
        &self.element
    }

    pub fn element(&self) -> &Element {
        &self.element
    }
}

pub fn get_dom_element_by_id(id: &str) -> Option<DomElement> {
    if let Some(el) = DOCUMENT.with(|d| d.get_element_by_id(id)) {
        return Some(DomElement::from(el));
    }
    None
}

///
/// attribute & class related
///
impl DomElement {
    // TODO check that the attribute *actually* was changed

    pub fn set_attribute(&self, key: &str, value: &str) {
        self.element
            .set_attribute(key, value)
            .expect("DomElement:set_attr()");
    }

    pub fn remove_attribute(&self, key: &str) {
        self.element
            .remove_attribute(key)
            .expect("DomElement:rm_attr()");
    }

    pub fn get_attribute(&self, key: &str) -> Option<String> {
        self.element.get_attribute(key)
    }

    pub fn has_class(&self, name: &str) -> bool {
        self.element.class_list().contains(name)
    }

    pub fn get_classes(&self) -> DomTokenList {
        self.element.class_list()
    }

    pub fn set_class(&self, cn: &str) {
        self.element.class_list().add_1(cn).unwrap();
        //self.set_class_name(cn);  --> removes all previous classes, and writes only this one
    }

    pub fn add_class(classes: &DomTokenList, value: &str) {
        classes.add_1(value).unwrap();
    }

    pub fn remove_class(classes: &DomTokenList, value: &str) {
        classes.remove_1(value).unwrap();
    }

    pub fn toggle_class(classes: &DomTokenList, value: &str) {
        classes.toggle(value).expect("DomElement:toggle_class()");
    }

    pub fn remove_class_starts_with(classes: &DomTokenList, start_string: &str) {
        for i in 0..classes.length() {
            let c: String = classes.get(i).unwrap();
            if c.starts_with(start_string) {
                classes.remove_1(&c).unwrap();
                return;
            }
        }
    }

    //Styles are <html_el style="key_1:value_1;key_2:value_2;" > ... </html_el>
    pub fn remove_style(element: &DomElement, style_key: &str) {
        let mut new_style = String::new();
        if let Some(el_style) = element.get_attribute("style") {
            for s in el_style.split(';') {
                if !s.starts_with(style_key) && !s.is_empty() {
                    new_style = [new_style, s.to_string(), ";".to_string()].concat();
                }
            }
        }
        if !new_style.is_empty() {
            element.remove_attribute("style");
            element.set_attribute("style", &new_style);
        }
    }

    //Styles are <html_el style="key_1:value_1;key_2:value_2;" > ... </html_el>
    pub fn add_style(element: &DomElement, style_key: &str, val: &str) {
        let mut new_style = String::new();
        let mut found = false;
        if let Some(el_style) = element.get_attribute("style") {
            for stl in el_style.split(';') {
                let v: Vec<&str> = stl.split(':').collect();
                if v.first().unwrap() == &style_key {
                    found = true;
                    new_style = [
                        new_style,
                        style_key.to_string(),
                        ":".to_string(),
                        val.to_string(),
                        ";".to_string(),
                    ]
                    .concat();
                } else if !stl.is_empty() {
                    new_style = [new_style, stl.to_string(), ";".to_string()].concat();
                }
            }
        }
        if !found {
            new_style = [
                new_style,
                style_key.to_string(),
                ":".to_string(),
                val.to_string(),
                ";".to_string(),
            ]
            .concat();
        }

        if !new_style.is_empty() {
            element.remove_attribute("style");
            element.set_attribute("style", &new_style);
        }
    }
}

///
/// visualisation related
///
impl DomElement {
    pub fn focus(&self) {
        self.element
            .clone()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap()
            .focus()
            .expect("can not focus");
    }

    pub fn blur(elem: &HtmlElement) {
        elem.blur().unwrap();
    }
}

///
/// element - node tree
///
impl DomElement {
    pub fn child_count(&self) -> usize {
        let child_nodes = self.element.child_nodes();
        child_nodes.length() as usize
    }

    pub fn insert_child_before(&self, child: &Node, other: &Node) {
        self.element.insert_before(child, Some(other)).unwrap();
    }

    pub fn insert_child(&self, index: usize, node: &Node) {
        let next_sibling = self.get_child(index);
        if let Some(next) = next_sibling {
            self.insert_child_before(node, &next);
        } else {
            self.append_child(node);
        }
    }

    //Child is the child being replaced; Other is the to be inserted one
    pub fn replace_child(&self, child: &Node, other: &Node) {
        self.element
            .replace_child(other, child)
            .expect("DomElement:replace_child()");
    }

    pub fn append_child(&self, child: &Node) {
        self.element
            .append_child(child)
            .expect("DomElement:append_child()");
    }

    pub fn remove_child(&self, child: &Node) {
        self.element
            .remove_child(child)
            .expect("DomElement:remove_child()");
    }

    pub fn remove_child_index(&self, index: usize) {
        let child = self.get_child(index);
        if child.is_none() {
            return;
        }
        self.remove_child(&child.unwrap());
    }

    // Return the last child node of the specified parent element (or `None`).

    pub fn get_last_child(&self) -> Option<Node> {
        let child_nodes = self.element.child_nodes();
        let child_count = child_nodes.length();
        if child_count == 0 {
            return None;
        }
        Some(
            child_nodes
                .get(child_count - 1)
                .expect("Could not access last child node"),
        )
    }

    // Return the last child node of the specified parent element (or `None`).

    pub fn get_child(&self, index: usize) -> Option<Node> {
        let i = index as u32;
        let child_nodes = self.element.child_nodes();
        let child_count = child_nodes.length();
        if i >= child_count {
            return None;
        }
        if child_count == 0 {
            return None;
        }
        Some(child_nodes.get(i).expect("Could not access child node"))
    }

    pub fn get_children(&self) -> NodeList {
        self.element.child_nodes()
    }

    pub fn get_parent(&self) -> Option<Element> {
        self.element.parent_element()
    }

    //Removes the "self" from the parent that contains it

    pub fn remove_child_from_parent(&self) {
        let parent_o = self.get_parent();
        if let Some(parent) = parent_o {
            let p_dom = DomElement::from(parent);
            p_dom.remove_child(self.node());
        }
    }

    //Find first matching child
    //searching depth first, but stopping at first match.

    pub fn find_down(&self, selector: &str) -> Option<Element> {
        let children = self.get_children();
        for i in 0..children.length() {
            let c = children.get(i).unwrap();
            if c.node_name() == selector {
                let el: Element = c.dyn_into::<Element>().unwrap();
                return Some(el);
            }
            if c.child_nodes().length() > 0 {
                let el: Element = c.dyn_into::<Element>().unwrap();
                let res: Option<Element> = DomElement::from(el).find_down(selector);
                if res.is_some() {
                    return res;
                }
            }
        }
        None
    }

    //Find (closest=) first matching parent
    //If we try to find a "DIV" starting from the "DIV" we get "Self" back, hence we start from the parents

    pub fn find_up(&self, selector: &str) -> Option<Element> {
        if let Some(el) = self.get_parent() {
            el.closest(selector)
                .expect("find_up(): expected query selector")
        } else {
            None
        }
    }

    //Will append to the new parent

    pub fn move_children_to_new_parent(&self, new_parent: &DomElement) {
        let children = self.get_children();
        let len = children.length();
        if len == 0 {
            return;
        }
        //tricky ... we have to remove element zero every time !!
        for _i in 0..len {
            let c = children.get(0).unwrap();
            self.remove_child(&c);
            new_parent.append_child(&c);
        }
    }

    //Will insert to the new parent

    pub fn move_to_new_parent(&self, index: usize, new_parent: &DomElement) {
        let parent = self.get_parent();
        if let Some(p) = parent {
            p.remove_child(self.node())
                .expect("DomElement:move_to_new_parent()"); //note: P is Element type, not DomElement
        }
        new_parent.insert_child(index, self.node());
    }
}
