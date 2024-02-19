// Copyright 2024 quill-core-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::error::Error::{CanNotRemoveASoftBreak, DoubleInsertionOfASoftBreak};
use anyhow::Result;
use dom::dom_element::DomElement;
use log::error;
use node_tree::doc_node::DocumentNode;
use std::sync::Arc;

///HTML tag
static SOFT_BREAK_TAG: &'static str = "BR";

/// # AutomaticSoftBreak
///
/// Tag which belongs to the HTML element `<BR/>` must be empty !! <br>
/// So we treat this format as a line format not a BLOCK format.
///
/// We have 2 soft break incarnations:
///  - Normal soft break which lives in a DeltaOperation document
///  - Automatically inserted (temporary) place holders to display an empty block node
///
/// The automatically inserted place holders shall have empty delta, and are NOT recognized
/// when inserted as a normal delta operation:
/// ```bash
///  Insert{
///    ""
/// }
///
/// The normal page break shows as a character of length 1
/// when inserted as a normal delta operation:
/// ```bash
///  Insert{
///    {"page_break", "true"}
/// }
/// ```
///
/// To ensure sane code using this functionality, we have panics if we detect wrong use ...
pub struct AutomaticSoftBreak {}

impl AutomaticSoftBreak {
    /// Uses DomElement manipulations not TreeMorph
    pub fn insert(doc_node: &Arc<DocumentNode>) -> Result<()> {
        //not very nice, but saves quite some tests in the op_transform module

        //In some cases we create a new node with a child
        let dn = if doc_node.child_count() == 1 {
            assert!(!doc_node
                .get_child(0)
                .unwrap()
                .get_formatter()
                .is_text_format());
            doc_node.get_child(0).unwrap()
        } else {
            doc_node.clone()
        };
        assert_eq!(dn.child_count(), 0);

        if AutomaticSoftBreak::has_break(&dn) {
            // supports debugging ...
            error!("AutomaticSoftBreak - you tried to insert an automatic break twice ...");
            error!(
                "AutomaticSoftBreak - FAIL to insert <BR>: {:?}",
                dn.get_operation()
            );
            return Err(DoubleInsertionOfASoftBreak.into());
        }
        let sb = DomElement::new(SOFT_BREAK_TAG);
        dn.get_dom_element().unwrap().insert_child(0, sb.node());
        Ok(())
    }

    /// The soft break shall be the first child of the parent.
    /// We do a quick check on length 0
    pub fn remove(doc_node: &Arc<DocumentNode>) -> Result<()> {
        assert!(!doc_node.get_formatter().is_text_format());
        let element = doc_node.get_dom_element().unwrap();
        if let Some(child) = element.get_child(0) {
            assert_eq!(child.node_name(), SOFT_BREAK_TAG);
            element.remove_child(&child);
        } else {
            // supports debugging ...
            error!(
                "AutomaticSoftBreak - FAIL to remove <BR>: {:?}",
                doc_node.get_operation()
            );
            return Err(CanNotRemoveASoftBreak.into());
        }
        Ok(())
    }

    fn has_break(doc_node: &Arc<DocumentNode>) -> bool {
        assert!(!doc_node.get_formatter().is_text_format());
        let parent = doc_node.get_dom_element().unwrap();
        if let Some(child) = parent.get_child(0) {
            if child.node_name() == SOFT_BREAK_TAG {
                return true;
            }
        }
        return false;
    }
}
