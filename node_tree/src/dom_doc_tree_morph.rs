// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//==============================================================================================
//Tree transformation functions are functions without "Self".
//They often need the "Self" to be an &Arc<DocumentNode> instead of an &DocumentNode (==&Self)
//We collected these in a separate module here
//==============================================================================================

use crate::doc_node::DocumentNode;
use crate::dom_doc_node::DomDocNode;
use std::sync::Arc;

/// # unlink()
///
/// Unlinks both the DocumentNode, and HTML dom elements from its parent
pub fn unlink(parent: &Arc<DocumentNode>, child: &Arc<DocumentNode>) {
    match &child.get_doc_dom_node() {
        DomDocNode::ElementNode(el) => {
            el.remove_child_from_parent();
        }
        DomDocNode::TextNode(el) => {
            el.rm_child_from_parent();
        }
    }
    remove_child(parent, child);
}

/// # append()
///
/// Appends a DocumentNode and links(appends) the HTML Dom node too
pub fn append(parent: &Arc<DocumentNode>, child: Arc<DocumentNode>) {
    //link html-dom
    match &parent.get_doc_dom_node() {
        DomDocNode::TextNode(_) => {
            panic!("You can not add a dom element to a text node");
        }
        DomDocNode::ElementNode(e) => {
            e.append_child(child.get_html_node());

            //link document node
            *child.parent.borrow_mut() = Arc::downgrade(parent);
            parent.children.borrow_mut().push(child);
        }
    }
}

/// # insert_before()
///
/// Inserts a a child before some other sibling of a parent node.
pub fn insert_before(
    parent: &Arc<DocumentNode>,
    next_sibling: &Arc<DocumentNode>,
    child: Arc<DocumentNode>,
) {
    let idx = parent.get_child_index(next_sibling);
    match idx {
        None => append(parent, child),
        Some(i) => insert_at_index(parent, i, child),
    }
}

/// # insert_at_index()
///
/// Inserts a DocumentNode and links(appends) the HTML Dom node too in the same index position
pub fn insert_at_index(parent: &Arc<DocumentNode>, index: usize, child: Arc<DocumentNode>) {
    //link html-dom
    match &parent.get_doc_dom_node() {
        DomDocNode::TextNode(_) => {
            panic!("You can not add a dom element to a text node");
        }
        DomDocNode::ElementNode(e) => {
            if index + 1 > parent.children.borrow().len() {
                append(parent, child);
            } else {
                e.insert_child(index, child.get_html_node());
                parent.children.borrow_mut().insert(index, child.clone());
                *child.parent.borrow_mut() = Arc::downgrade(parent);
            }
        }
    }
}

/// # insert_after()
///
/// Inserts a new DocumentNode after the child designated as `child_left`.
pub fn insert_after(
    parent: &Arc<DocumentNode>,
    child_left: &Arc<DocumentNode>,
    new_child: &Arc<DocumentNode>,
) {
    let index = parent.get_child_index(child_left).unwrap();
    insert_at_index(parent, index + 1, new_child.clone());
}

/// # remove_child_index()
///
/// Removes the child found at some index of a parent
pub fn remove_child_index(parent: &Arc<DocumentNode>, index: usize) {
    let child = parent.get_child(index);
    match child {
        None => (),
        Some(c) => {
            parent.children.borrow_mut().remove(index);
            unlink(parent, &c);
        }
    }
}

/// # remove_child()
///
/// Removes a child from a parent. When the child is not a child, then we ignore; no error.
pub fn remove_child(parent: &Arc<DocumentNode>, child: &Arc<DocumentNode>) {
    match parent.get_child_index(child) {
        None => {}
        Some(index) => remove_child_index(parent, index),
    }
}
