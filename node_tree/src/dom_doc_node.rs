// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::doc_node::DocumentNode;
use crate::tree_traverse::{first_node, next_node};
use dom::dom_element::DomElement;
use dom::dom_text::{DomText, TEXT_TAG};
use log::debug;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node, Text};

/// # DomDocNode
///
/// We are appending 2 kinds of elements: TEXT and ELEMENT
///
/// Both are specialisations of the trait NODE; but we do not want to lose the Trait type here
/// DDM parent child relations only know NODE;
/// We add aa enum here with the type of element
#[derive(Debug)]
pub enum DomDocNode {
    TextNode(DomText),
    ElementNode(DomElement),
}

impl From<Node> for DomDocNode {
    fn from(n: Node) -> Self {
        debug!("DocDomNode::from() --> node_name = {:?} ", n.node_name());
        if n.node_name() == *TEXT_TAG {
            let t = n.dyn_into::<Text>().unwrap();
            let tn = DomText::from(t);
            DomDocNode::TextNode(tn)
        } else {
            let e = n.dyn_into::<Element>().unwrap();
            let en = DomElement::from(e);
            DomDocNode::ElementNode(en)
        }
    }
}

impl DomDocNode {
    pub fn get_node(&self) -> &Node {
        match &self {
            DomDocNode::ElementNode(e) => e.node(),
            DomDocNode::TextNode(n) => n.node(),
        }
    }
    pub fn get_node_name(&self) -> String {
        self.get_node().node_name()
    }

    pub fn is_text(&self) -> bool {
        match self {
            DomDocNode::TextNode(_) => true,
            DomDocNode::ElementNode(_) => false,
        }
    }

    pub fn is_element(&self) -> bool {
        !self.is_text()
    }
}

impl From<Element> for DomDocNode {
    fn from(n: Element) -> Self {
        let el = DomElement::from(n);
        DomDocNode::ElementNode(el)
    }
}

impl From<DomElement> for DomDocNode {
    fn from(n: DomElement) -> Self {
        DomDocNode::ElementNode(n)
    }
}

impl From<DomText> for DomDocNode {
    fn from(n: DomText) -> Self {
        DomDocNode::TextNode(n)
    }
}

pub fn find_doc_node_from_text_node(
    node: &Node,
    root: &Arc<DocumentNode>,
) -> Option<Arc<DocumentNode>> {
    assert!(node.node_type() == Node::TEXT_NODE);
    if let Some(doc_node) = find_doc_node(node, root) {
        return Some(doc_node);
    }
    //error!( "selected_node::find_doc_node_from_text_node() ...So we are looking for my parent.");
    let mut parent: Node = node.clone();
    loop {
        parent = parent.parent_node().unwrap();
        if let Some(doc_node) = find_doc_node(parent.as_ref(), root) {
            return Some(doc_node);
        }
        if parent.node_name() == "DIV" {
            panic!( "selected_node::find_doc_node_from_text_node() ...Looks like we found root element.");
        }
    }
}

pub fn find_doc_node_from_element_node(
    node: &Node,
    root: &Arc<DocumentNode>,
) -> Option<Arc<DocumentNode>> {
    assert_eq!(node.node_type(), Node::ELEMENT_NODE);
    if let Some(doc_node) = find_doc_node(node, root) {
        return Some(doc_node);
    }
    panic!("selected_node::find_doc_node_from_element_node() ...nothing found.");
}

fn find_doc_node(node: &Node, root: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    //FIXME: There are probably smarter ways to get to this node ...
    // let html = dom_print::pretty_print(node, true );
    // error!( "selected_node::find_doc_node() - Start node:\n{}", doc_node);
    let mut dn = Some(first_node(root));
    while dn.is_some() {
        let doc_node = dn.unwrap();
        if doc_node.get_html_node().eq(node) {
            debug!(
                "selected_node::find_doc_node() - \n{:?}",
                doc_node.get_operation()
            );
            return Some(doc_node);
        }
        dn = next_node(&doc_node);
    }
    debug!("selected_node::find_doc_node() ...nothing found.");
    None
}
