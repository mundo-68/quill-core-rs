// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::doc_node::DocumentNode;
use crate::EDITOR_CLASS;
use std::sync::Arc;

/// # tree_traverse
///
/// Iterator methods to traverse the tree in a LINEAR order of the document-transformations
/// The doc-node tree may look like below. Since we have the "block" operations AFTER the
///  declaration of the leaf node, we should return the block B after E, which comes after D.
/// ```bash
///       A
///      / \
///     B   C
///    / \ / \
///   D  E F  G
///```
/// Repeated calls to `Next()` should return :D, E, B, F, G, C, A
///
/// Debug hit: If there are `*.unwrap()` errors from this library, then it is probably a
/// `unlinked-node` that we use as input.
///
/// Maybe have a look at the traversal package:
/// [DftPost](https://docs.rs/traversal/0.1.2/traversal/struct.DftPost.html]) (Depth-First Traversal in Post-Order)

//==========================================================
// Support functions not part of document impl ...
//==========================================================
/// Root element of an editable tree is marked by a `<DIV>` with non empty ID attribute
pub fn is_doc_root(doc_node: &DocumentNode) -> bool {
    if let Some(el) = doc_node.get_dom_element() {
        let el_type = el.node_name();
        if el_type == "DIV" && el.has_class(EDITOR_CLASS) {
            return true;
        }
    }
    false
}

/// Finds the root of a  document given any child node
pub fn get_root(doc_node: &Arc<DocumentNode>) -> Arc<DocumentNode> {
    if is_doc_root(doc_node) {
        return doc_node.clone();
    }
    let mut parent = doc_node.get_parent().unwrap();
    loop {
        if is_doc_root(&parent) {
            return parent.clone();
        }
        parent = if let Some(p) = parent.get_parent() {
            p
        } else {
            panic!("Hey it seems we can not find the root node ...");
        }
    }
}

/// Move to the first node of a document. If the document is empty we will
/// get a `<P>` block back.
/// The minimum content of a document is at least `<P></P>`
/// and Text(hello world) is the first leaf of `<P>hello world</P>`
//FIXME: Remove the Option<> from the output
pub fn first_node(node: &Arc<DocumentNode>) -> Arc<DocumentNode> {
    let root = get_root(node);
    //assert!(is_doc_root(root));
    let c = root.get_children();
    if !c.is_empty() {
        return first_iter_intern(c.first().unwrap());
    }
    panic!("there must be a first node which is NOT the root");
}
#[inline(always)]
fn first_iter_intern(doc_node: &Arc<DocumentNode>) -> Arc<DocumentNode> {
    let c = doc_node.get_children();
    if !c.is_empty() {
        return first_iter_intern(c.first().unwrap());
    }
    doc_node.clone()
}

pub fn last_block_node(root: &DocumentNode) -> Option<Arc<DocumentNode>> {
    assert!(is_doc_root(root));
    let len = root.child_count();
    assert!(len > 0); //minimum content of a document is at least <P></P>
    root.get_child(len - 1)
}

/// This is the last node in a non empty document, but not the last NODE (block node follows)
//FIXME: Remove the Option<> from the output
pub fn last_leaf_node(root: &DocumentNode) -> Option<Arc<DocumentNode>> {
    assert!(is_doc_root(root));
    let c = root.get_children();
    let l = c.len();
    if l > 0 {
        return last_iter_intern(c.get(l - 1).unwrap());
    }
    panic!("there must be a last node which is NOT the root");
}
#[inline(always)]
fn last_iter_intern(doc_node: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let c = doc_node.get_children();
    let l = c.len();
    if l > 0 {
        return last_iter_intern(c.get(l - 1).unwrap());
    }
    Some(doc_node.clone())
}

/// # next_node()
///
/// Moves from node to node
///
/// Iterator methods to traverse the tree in a LINEAR order of the document-transformations
/// The doc-node tree may look like below. Since we have the "block" operations AFTER the
///  declaration of the leaf node, we should return the block B after E, which comes after D.
/// ```bash
///       A
///      / \
///     B   C
///    / \ / \
///   D  E F  G
///```
/// Repeated calls to `Next()` should return :D, E, B, F, G, C, A
/// We do not return root element
pub fn next_node(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    if let Some(parent) = current.get_parent() {
        let children = parent.get_children();
        let my_index = parent.get_child_index(current).unwrap();
        if my_index + 1 < children.len() {
            return Some(first_iter_intern(&parent.get_child(my_index + 1).unwrap()));
        }
        if my_index + 1 == children.len() && !is_doc_root(&parent) {
            return Some(parent);
        }
    }
    None
}

/// FIXME: Should this be the normal traverse function?
/// Why use traversing and showing the "dummy" blocks
/// We should not need them in a sense that the "isolate()" call will cut the dummy structures
/// such that the tree manipulations will do what ever you would expect ...
pub fn next_node_non_zero_length(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let mut cur = current.clone();
    loop {
        if let Some(nxt) = next_node(&cur) {
            if nxt.op_len() > 0 {
                return Some(nxt);
            } else {
                cur = nxt;
            }
        } else {
            return None;
        }
    }
}

/// # prev_node()
///
/// Get the previous document node.
///
/// Iterator methods to traverse the tree in a backwards LINEAR order of the document-transformations
/// ```bash
///      A
///     / \
///    B   C
///   / \ / \
///  D  E F  G
/// ```
/// Repeated calls to `Prev()` should return: (A), C, G, F, B, E, D
///
/// 1) Keep going to the last child, until that child is not a leaf anymore
/// 2) If you are not the last sibling, go to the previous sibling
/// 3) If you are the first sibling, and the parent == root --> you are done
/// 4) if not 3) then, go to the parent-parent previous sibling, and try 2) and 3) again
///
/// Moves from node to node, depth first as shown in the header text of this file
pub fn prev_node(doc_node: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    //rule 0)
    if is_doc_root(doc_node) {
        return None;
    }

    //Rule 1)
    if doc_node.child_count() > 0 {
        let children = doc_node.get_children();
        let node = children.last().unwrap();
        return Some(node.clone());
    }

    let mut parent_o = doc_node.get_parent();
    let mut child = doc_node.clone();
    loop {
        if let Some(parent) = parent_o {
            let my_index = parent.get_child_index(&child).unwrap();
            //rule 2)
            if my_index > 0 {
                return parent.get_child(my_index - 1);
            }

            //rule 3)
            if is_doc_root(&parent) {
                return None;
            }

            //rule 4), and apply rule 2) again
            parent_o = parent.get_parent();
            child = parent;
        } else {
            panic!( "hey I can not find a parent, but I also have not seen the root node of the document!!");
        }
    }
}

pub fn prev_node_non_zero_length(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let mut cur = current.clone();
    loop {
        if let Some(prv) = prev_node(&cur) {
            if prv.op_len() > 0 {
                return Some(prv);
            } else {
                cur = prv; //next iteration
            }
        } else {
            return None;
        }
    }
}

/// Moves from block to block, skipping leaf nodes
///
/// Blocks of zero length are skipped. These are specials, and should not be
/// considered a "real" block node.
pub fn next_block(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let mut next = current.clone();
    loop {
        if let Some(p) = next_node(&next) {
            if !p.get_formatter().is_text_format() {
                if p.op_len() == 0 {
                    //skip blocks with length 0 they are only there for visualisation ..
                    next = p;
                } else {
                    return Some(p);
                }
            } else {
                next = p;
            }
        } else {
            return None;
        }
    }
}

/// Moves from block to block, skipping leaf nodes
pub fn prev_block(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let mut prev = current.clone();
    loop {
        if let Some(p) = prev_node(&prev) {
            if !p.get_formatter().is_text_format() {
                if p.op_len() == 0 {
                    //skip blocks with length 0 they are only there for visualisation ..
                    prev = p;
                } else {
                    return Some(p);
                }
            } else {
                prev = p;
            }
        } else {
            return None;
        }
    }
}

/// Moves from leaf to leaf, skipping block nodes
pub fn next_leaf(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let mut next = current.clone();
    loop {
        if let Some(p) = next_node(&next) {
            if p.get_formatter().is_text_format() {
                return Some(p);
            } else {
                next = p;
            }
        } else {
            return None;
        }
    }
}

/// Moves from leaf to leaf, skipping block nodes
pub fn prev_leaf(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let mut prev = current.clone();
    loop {
        if let Some(p) = prev_node(&prev) {
            if p.get_formatter().is_text_format() {
                return Some(p);
            } else {
                prev = p;
            }
        } else {
            return None;
        }
    }
}

//Returns a next sibling of a given parent;  of none if the parent does not match
pub fn next_sibling(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    assert!(!is_doc_root(current));
    let p = current.get_parent().unwrap();
    let my_index = p.get_child_index(current).unwrap();
    if my_index + 1 < p.child_count() {
        p.get_child(my_index + 1)
    } else {
        None
    }
}

/// Returns a previous sibling of a given parent;
/// or none if the parent does not match
pub fn prev_sibling(current: &Arc<DocumentNode>) -> Option<Arc<DocumentNode>> {
    let p = current.get_parent().unwrap();
    let my_index = p.get_child_index(current).unwrap();
    if my_index > 0 {
        p.get_child(my_index - 1)
    } else {
        None
    }
}
