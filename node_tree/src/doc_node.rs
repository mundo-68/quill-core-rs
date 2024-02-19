// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::dom_doc_node::DomDocNode;
use crate::format_trait::FormatTait;
use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use dom::dom_text;
use dom::dom_text::DomText;
use std::cell::RefCell;
use std::ops::Deref;
use std::ptr;
use std::sync::{Arc, Weak};
use web_sys::Node;

pub struct DocumentNode {
    formatter: RefCell<Arc<dyn FormatTait + Sync + Send>>,
    element: DomDocNode,
    delta_op: RefCell<DeltaOperation>,
    pub(crate) children: RefCell<Vec<Arc<DocumentNode>>>,
    pub(crate) parent: RefCell<Weak<DocumentNode>>,
}

//================================================================
// Data structure definitions
//================================================================
pub type DnodePtr = Arc<DocumentNode>;

impl PartialEq for DocumentNode {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(&self, &other)
    }
}
impl Eq for DocumentNode {}

//================================================================
// Creators
//================================================================
impl DocumentNode {
    pub fn new_node(node: Node, formatter: Arc<dyn FormatTait + Send + Sync>) -> Self {
        let d = DomDocNode::from(node);
        DocumentNode {
            formatter: RefCell::new(formatter),
            element: d,
            delta_op: RefCell::new(DeltaOperation::insert("")),
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(Weak::new()),
        }
    }

    pub fn new_element(
        dom_element: DomElement,
        formatter: Arc<dyn FormatTait + Send + Sync>,
    ) -> Self {
        DocumentNode {
            formatter: RefCell::new(formatter),
            element: DomDocNode::ElementNode(dom_element),
            delta_op: RefCell::new(DeltaOperation::insert("")),
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(Weak::new()),
        }
    }

    pub fn new_text(dom_element: DomText, formatter: Arc<dyn FormatTait + Send + Sync>) -> Self {
        DocumentNode {
            formatter: RefCell::new(formatter),
            element: DomDocNode::TextNode(dom_element),
            delta_op: RefCell::new(DeltaOperation::insert("")),
            children: RefCell::new(Vec::new()),
            parent: RefCell::new(Weak::new()),
        }
    }
}

//================================================================
// Operations on the document node
//================================================================
impl DocumentNode {
    /// # op_len()
    ///
    /// Returns the length of this operation WITHOUT child length include. Hence: Just the operation!
    ///
    /// This function is not called "len()" or "length()" contrary to conventions to emphasize the difference
    pub fn op_len(&self) -> usize {
        self.delta_op.borrow().op_len()
    }

    pub fn child_count(&self) -> usize {
        return self.children.borrow().len();
    }

    pub fn set_operation(&self, op: DeltaOperation) {
        *self.delta_op.borrow_mut() = op;
    }

    pub fn get_operation(&self) -> DeltaOperation {
        self.delta_op.borrow().clone()
    }

    //FIXME: Drop this, use .is_text_format()
    pub fn is_text(&self) -> bool {
        self.formatter.borrow().is_text_format()
    }
    //FIXME: Drop this, use .is_text_format()
    pub fn is_leaf(&self) -> bool {
        self.formatter.borrow().is_text_format()
    }

    /// # get_formatter()
    ///
    /// A formatter is the implementation that handles the `format_trait` interface for the
    /// `DeltaOperation` which is attached to this document node
    pub fn get_formatter(&self) -> Arc<dyn FormatTait + Send + Sync> {
        self.formatter.borrow().deref().clone()
    }

    /// # set_formatter()
    ///
    /// A formatter is the implementation that handles the `format_trait` interface for the
    /// `DeltaOperation` which is attached to this document node
    pub fn set_formatter(&self, t: &Arc<dyn FormatTait + Send + Sync>) {
        *self.formatter.borrow_mut() = t.clone();
    }

    pub fn get_doc_dom_node(&self) -> &DomDocNode {
        &self.element
    }

    /// # get_dom_element()
    ///
    /// If this `DocDomNode` contains an `Element` it is returned. None otherwise.
    pub fn get_dom_element(&self) -> Option<&DomElement> {
        match &self.element {
            DomDocNode::TextNode(_) => None,
            DomDocNode::ElementNode(el) => Some(el),
        }
    }

    /// # get_dom_text()
    ///
    /// If this `DocDomNode` contains an `TEXT` it is returned. None otherwise.
    pub fn get_dom_text(&self) -> Option<&DomText> {
        match &self.element {
            DomDocNode::ElementNode(_) => None,
            DomDocNode::TextNode(el) => Some(el),
        }
    }

    pub fn get_html_node(&self) -> &Node {
        match &self.element {
            DomDocNode::TextNode(txt) => txt.node(),
            DomDocNode::ElementNode(el) => el.node(),
        }
    }

    /// # find_dom_text()
    ///
    /// Returns the DomText;
    ///
    /// Implementation note: that some "leaf" nodes still are HTML-element nodes (link, span, ...)
    pub fn find_dom_text(&self) -> DomText {
        dom_text::find_dom_text(self.get_html_node()).unwrap()
    }

    /// # get_parent()
    ///
    /// We define the parent here as the parent in the tree. This may or may not
    /// be a parent that is a virtual node with `op_len() == 0` or a real document node with length
    /// more than zero.
    ///
    /// If the parent is needed skipping any virtual nodes use: `next_block(&some_child_node)`.
    /// Then the first parent node is returned that is a real delta_operation, not a virtual one.
    pub fn get_parent(&self) -> Option<Arc<DocumentNode>> {
        self.parent.borrow().upgrade()
    }

    /// # get_children()
    ///
    /// Returns a vector of children.
    ///
    /// Implementation note: Only children are reported which are in the DocumentNode structure.
    /// It may be that the `formatter_trait` implementation adds DOM nodes. Example: add chapter number
    /// to a chapter title.
    pub fn get_children(&self) -> Vec<Arc<DocumentNode>> {
        self.children.borrow().clone()
    }

    /// # get_child()
    ///
    /// Returns one child out of the vector of children. We return the requested vector index.
    ///
    /// Implementation note: Only children are reported which are in the DocumentNode structure.
    /// It may be that the `formatter_trait` implementation adds DOM nodes. Example: add chapter number
    /// to a chapter title.
    pub fn get_child(&self, index: usize) -> Option<Arc<DocumentNode>> {
        let len = self.children.borrow().len();
        if len == 0 || index + 1 > len {
            return None;
        }
        return Some(self.children.borrow().get(index).unwrap().clone());
    }

    /// # get_child_index()
    ///
    /// If the child is in the vector of children for this node, the index in the vector is returned.
    pub fn get_child_index(&self, child: &Arc<DocumentNode>) -> Option<usize> {
        for (count, c) in self.children.borrow().iter().enumerate() {
            //if Arc::eq(c,child) {
            if ptr::eq(c.deref(), child.deref()) {
                return Some(count);
            }
        }
        None
    }

    /// # my_index_as_child()
    ///
    /// Checking the children of `self.parent`. If `self` as a child is in the vector of the `self.parent` children,
    /// the index in the vector is returned.
    pub fn my_index_as_child(&self) -> Option<usize> {
        let parent = self.get_parent().unwrap();
        for (count, child) in parent.children.borrow().iter().enumerate() {
            if ptr::eq(child.deref(), self) {
                return Some(count);
            }
        }
        None
    }

    pub fn is_empty_block(&self) -> bool {
        let is_block = !self.formatter.borrow().is_text_format();
        let is_empty = self.children.borrow().len() == 0;
        is_block && is_empty
    }
}

//-----------------------------------------------------------------------------
// Display fmt for debugging purposes ...
//-----------------------------------------------------------------------------
// impl Display for DocumentNode {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "DocumentNode->[\n\tNode: {}, \n\tOperation : {} \n]\n",
//                self.get_doc_dom_node().get_node_name(), self.get_operation().to_string()
//         )
//     }
// }

#[cfg(test)]
const TAB: &str = "  ";
#[cfg(test)]
impl std::fmt::Display for DocumentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = print_doc_node_intern(&self, TAB);
        write!(f, "{}", s)
    }
}

#[cfg(test)]
fn print_doc_node_intern(doc_node: &DocumentNode, tab: &str) -> String {
    let mut s = format!(
        "{}DocNode->[ Element: {:?}, Delta : {:?} ] \n",
        &tab,
        doc_node.get_doc_dom_node().get_node_name(),
        doc_node.get_operation()
    );
    let t: String = [&tab, TAB].concat();
    for c in doc_node.get_children().iter() {
        s = [s, print_doc_node_intern(c, &t)].concat();
    }
    return s;
}
